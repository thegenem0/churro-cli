use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use std::{collections::HashMap, convert::Infallible, net::SocketAddr, sync::Arc};

use dotenv_codegen::dotenv;
use eyre::Result;
use oauth2::{
    basic::{BasicClient, BasicTokenType},
    reqwest::async_http_client,
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EmptyExtraTokenFields,
    PkceCodeChallenge, RedirectUrl, Scope, StandardTokenResponse, TokenUrl,
};
use tokio::sync::{mpsc, Mutex};
use webbrowser;

const CLIENT_ID: &str = dotenv!("CLIENT_ID");
const CLIENT_SECRET: &str = dotenv!("CLIENT_SECRET");
const JIRA_AUTH_URI: &str = dotenv!("JIRA_AUTH_URI");
const JIRA_REDIRECT_URI: &str = dotenv!("JIRA_REDIRECT_URI");
const JIRA_TOKEN_URL: &str = dotenv!("JIRA_TOKEN_URL");

const JIRA_READ_USER_SCOPE: &str = dotenv!("JIRA_READ_USER_SCOPE");
const JIRA_READ_WORK_SCOPE: &str = dotenv!("JIRA_READ_WORK_SCOPE");
const JIRA_STATE: &str = dotenv!("JIRA_STATE");

pub async fn authenticate(
) -> Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>, eyre::Error> {
    let client = BasicClient::new(
        ClientId::new(CLIENT_ID.to_string()),
        Some(ClientSecret::new(CLIENT_SECRET.to_string())),
        AuthUrl::new(JIRA_AUTH_URI.to_string())?,
        Some(TokenUrl::new(JIRA_TOKEN_URL.to_string())?),
    )
    .set_redirect_uri(RedirectUrl::new(JIRA_REDIRECT_URI.to_string())?);

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_extra_param("audience", "https://api.atlassian.com")
        .add_extra_param("prompt", "consent")
        .add_extra_param("state", JIRA_STATE.to_string())
        .add_scope(Scope::new(JIRA_READ_USER_SCOPE.to_string()))
        .add_scope(Scope::new(JIRA_READ_WORK_SCOPE.to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    let (tx, rx) = mpsc::unbounded_channel();
    let tx_clone = tx.clone();
    let rx = Arc::new(Mutex::new(rx));

    let service = make_service_fn(move |_| {
        let tx = tx.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                handle_request(req, tx.clone())
            }))
        }
    });

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let server = Server::bind(&addr).serve(service);

    let rx_clone = Arc::clone(&rx);
    let server_with_shutdown = server.with_graceful_shutdown(async move {
        let _ = rx_clone.lock().await.recv().await;
    });
    tokio::spawn(server_with_shutdown);

    log::info!("Listening on http://{}", addr);

    webbrowser::open(&auth_url.to_string())?;

    let result = match Arc::clone(&rx).lock().await.recv().await {
        Some(result) => result,
        None => {
            return Err(eyre::eyre!("No result from auth server"));
        }
    };

    let _ = tx_clone.send("Shutdown".to_string());

    let token_result = client
        .exchange_code(AuthorizationCode::new(result))
        .set_pkce_verifier(pkce_verifier)
        .request_async(async_http_client)
        .await;
    if !token_result.is_err() {
        log::info!("Successful login");
    }

    Ok(token_result?)
}

async fn handle_request(
    req: Request<Body>,
    tx: mpsc::UnboundedSender<String>,
) -> Result<Response<Body>, Infallible> {
    if let Some(query) = req.uri().query() {
        let params: HashMap<_, _> = url::form_urlencoded::parse(query.as_bytes())
            .into_owned()
            .collect();

        if let Some(code) = params.get("code") {
            tx.send(code.to_string()).unwrap();
        }

        if let Some(error) = params.get("error") {
            let default_description = "No description".to_string();
            let error_description = params
                .get("error_description")
                .unwrap_or(&default_description);
            log::error!("Error: {}, Reason: {}", error, error_description);
            tx.send(format!("Error")).unwrap();
        }
    }
    Ok(Response::new(Body::empty()))
}

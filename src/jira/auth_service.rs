use base64::{engine::general_purpose, Engine as _};
use rand;
use rand::Rng;
use reqwest;
use sha2::{Digest, Sha256};
use url::Url;

const CLIENT_ID: &str = "srkIUYFtkY8FEEg5iqw4rX0qZCnaW4JD";
const REDIRECT_URI: &str = "https://localhost:8080";

// Step 1: Redirect users to request Jira Cloud access
pub fn build_authorization_url() -> Result<(Url, String), Box<dyn std::error::Error>> {
    // generate a new PKCE code_verifier and SHA256 encode it to create the code_challenge
    let code_verifier: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(50)
        .map(char::from)
        .collect();
    let code_challenge = Sha256::digest(code_verifier.as_bytes());

    // build the authorization URL
    let auth_url = Url::parse("https://auth.atlassian.com/authorize")?
        .query_pairs_mut()
        .append_pair("audience", "api.atlassian.com")
        .append_pair("client_id", CLIENT_ID)
        .append_pair("scope", "read:jira-user read:jira-work")
        .append_pair("redirect_uri", REDIRECT_URI)
        .append_pair("state", "aRandomState")
        .append_pair("response_type", "code")
        .append_pair("prompt", "consent")
        .append_pair("code_challenge_method", "S256")
        .append_pair(
            "code_challenge",
            &general_purpose::STANDARD.encode(code_challenge),
        )
        .finish()
        .to_owned();

    Ok((auth_url, code_verifier))
}

// Step 2: Users are redirected back to your site by Jira Cloud
pub async fn exchange_code_for_token(
    authorization_code: String,
    code_verifier: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let params = [
        ("grant_type", "authorization_code"),
        ("client_id", CLIENT_ID),
        ("code", &authorization_code),
        ("redirect_uri", REDIRECT_URI),
        ("code_verifier", &code_verifier),
    ];

    let req = client
        .post("https://auth.atlassian.com/oauth/token")
        .form(&params)
        .build()?;

    let res = client
        .post("https://auth.atlassian.com/oauth/token")
        .form(&params)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    println!("{:#?}", req);

    match res.get("access_token") {
        Some(token) => Ok(token.as_str().unwrap().to_string()),
        None => Err("access_token not found".into()),
    }
}

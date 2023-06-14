use reqwest;
use url::Url;

const CLIENT_ID: &str = "srkIUYFtkY8FEEg5iqw4rX0qZCnaW4JD";
const CLIENT_SECRET: &str =
    "ATOAtPgJf2eMRvO5os6zgfoguP3oI0bKN_Vpc7xWAU3xZyudfDlVpZ4jAmk4aNyx-IltD787E286";
const REDIRECT_URI: &str = "https://localhost/callback";

// Step 1: Redirect users to request Jira Cloud access
pub fn build_authorization_url() -> Result<Url, Box<dyn std::error::Error>> {
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
        .finish()
        .to_owned();

    Ok(auth_url)
}

// Step 2: Users are redirected back to your site by Jira Cloud
pub async fn exchange_code_for_token(
    authorization_code: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let params = [
        ("grant_type", "authorization_code"),
        ("client_id", CLIENT_ID),
        ("client_secret", CLIENT_SECRET),
        ("code", &authorization_code),
        ("redirect_uri", REDIRECT_URI),
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
    println!("{:#?}", res);

    match res.get("access_token") {
        Some(token) => Ok(token.as_str().unwrap().to_string()),
        None => Err("No access token found".into()),
    }
}

pub async fn get_accessible_scopes(
    access_token: &String,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let res = client
        .get("https://api.atlassian.com/oauth/token/accessible-resources")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Accept", "application/json")
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    println!("{:#?}", res);

    return Ok(res);
}

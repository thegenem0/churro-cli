use reqwest;
use serde::de::DeserializeOwned;

pub async fn get<T: DeserializeOwned>(url: String) -> Result<T, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let resp = client.get(&url).send().await?.json().await?;
    Ok(resp)
}

pub async fn post<T: DeserializeOwned>(
    url: String,
    body: String,
) -> Result<T, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let resp = client.post(&url).body(body).send().await?.json().await?;
    Ok(resp)
}

pub async fn put<T: DeserializeOwned>(
    url: String,
    body: String,
) -> Result<T, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let resp = client.put(&url).body(body).send().await?.json().await?;
    Ok(resp)
}

pub async fn delete<T: DeserializeOwned>(
    url: String,
    body: String,
) -> Result<T, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let resp = client.delete(&url).body(body).send().await?.json().await?;
    Ok(resp)
}

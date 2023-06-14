use reqwest;
use serde::de::DeserializeOwned;

pub struct JiraService {
    pub base_url: String,
    pub jwt: String,
}

impl JiraService {
    pub fn new(base_url: String, jwt: String) -> Self {
        Self { base_url, jwt }
    }

    pub async fn get<T: DeserializeOwned>(
        &self,
        query_path: &str,
    ) -> Result<T, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let url = format!("{}/{}", self.base_url, query_path);
        let resp = client
            .get(url)
            .header("Authorization", format!("Bearer {}", jwt))
            .send()
            .await?
            .json()
            .await?;
        Ok(resp)
    }

    pub async fn post<T: DeserializeOwned>(
        &self,
        query_path: String,
        body: String,
    ) -> Result<T, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let resp = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", jwt))
            .body(body)
            .send()
            .await?
            .json()
            .await?;
        Ok(resp)
    }

    pub async fn put<T: DeserializeOwned>(
        url: String,
        jwt: &String,
        body: String,
    ) -> Result<T, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let resp = client
            .put(&url)
            .header("Authorization", format!("Bearer {}", jwt))
            .body(body)
            .send()
            .await?
            .json()
            .await?;
        Ok(resp)
    }

    pub async fn delete<T: DeserializeOwned>(
        url: String,
        jwt: &String,
        body: String,
    ) -> Result<T, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let resp = client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", jwt))
            .body(body)
            .send()
            .await?
            .json()
            .await?;
        Ok(resp)
    }
}

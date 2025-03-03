use reqwest::{Client, Response};
use std::collections::HashMap;
use std::error::Error;

#[derive(Clone)]
pub struct HttpAdapter {
    client: Client,
    headers: HashMap<String, String>,
    base_url: String,
}

impl HttpAdapter {
    pub fn new(base_url: String) -> Self {
        HttpAdapter {
            client: Client::new(),
            headers: HashMap::new(),
            base_url,
        }
    }

    pub fn add_header(&mut self, key: &str, value: &str) {
        self.headers.insert(key.to_string(), value.to_string());
    }

    pub async fn get(&self, url: &str) -> Result<String, Box<dyn Error>> {
        let url_complete = format!("{}{}", self.base_url, url);
        let request = self.client.get(&url_complete);

        let response = self.send_request(request).await?;
        Ok(response)
    }

    pub async fn post(&self, url: &str, body: Option<&str>) -> Result<String, Box<dyn Error>> {
        let url_complete = format!("{}{}", self.base_url, url);
        let request = self.client.post(&url_complete);

        let request = if let Some(b) = body {
            request.body(b.to_string())
        } else {
            request
        };

        let response = self.send_request(request).await?;
        Ok(response)
    }

    pub async fn put(&self, url: &str, body: Option<&str>) -> Result<String, Box<dyn Error>> {
        let url_complete = format!("{}{}", self.base_url, url);
        let request = self.client.put(&url_complete);

        let request = if let Some(b) = body {
            request.body(b.to_string())
        } else {
            request
        };

        let response = self.send_request(request).await?;
        Ok(response)
    }

    pub async fn delete(&self, url: &str) -> Result<String, Box<dyn Error>> {
        let url_complete = format!("{}{}", self.base_url, url);
        let request = self.client.delete(&url_complete);

        let response = self.send_request(request).await?;
        Ok(response)
    }

    async fn send_request(&self, request: reqwest::RequestBuilder) -> Result<String, Box<dyn Error>> {
        let request = self.apply_headers(request);
        let response: Response = request.send().await?;

        if !response.status().is_success() {
            return Err(format!("Failed to send request, status code: {}", response.status()).into());
        }

        let body = response.text().await?;

        Ok(body)
    }

    fn apply_headers(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        let mut request = request;
        for (key, value) in &self.headers {
            request = request.header(key, value);
        }
        request
    }
}
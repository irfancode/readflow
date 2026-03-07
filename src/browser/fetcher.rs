use anyhow::{Context, Result};
use reqwest::{Client, header::{HeaderMap, HeaderName, HeaderValue, ACCEPT, ACCEPT_LANGUAGE}};
use std::time::Duration;
use tracing::{debug, info};

pub struct Fetcher {
    client: Client,
}

impl Fetcher {
    pub fn new() -> Result<Self> {
        Self::new_with_verify(true)
    }

    pub fn new_with_verify(verify_ssl: bool) -> Result<Self> {
        let mut builder = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .default_headers({
                let mut headers = HeaderMap::new();
                headers.insert(ACCEPT, HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"));
                headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));
                headers
            })
            .cookie_store(true);

        if !verify_ssl {
            builder = builder.danger_accept_invalid_certs(true);
        }

        let client = builder
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { client })
    }

    pub async fn fetch(&self, url: &str) -> Result<Response> {
        info!("Fetching: {}", url);
        
        let response = self.client
            .get(url)
            .send()
            .await
            .context("Failed to send request")?;

        let status = response.status();
        let headers: Vec<(String, String)> = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        debug!("Response status: {} for {}", status, url);

        let body = response
            .bytes()
            .await
            .context("Failed to read response body")?;

        let charset = detect_charset(&headers, &body);
        let text = decode_body(&body, &charset);

        Ok(Response {
            url: url.to_string(),
            status: status.as_u16(),
            headers,
            body: text,
            raw_bytes: body.to_vec(),
        })
    }

    pub async fn fetch_with_headers(&self, url: &str, custom_headers: &[(&str, &str)]) -> Result<Response> {
        let mut request = self.client.get(url);
        
        for (key, value) in custom_headers {
            if let (Ok(name), Ok(val)) = (
                HeaderName::try_from(*key),
                HeaderValue::from_str(value)
            ) {
                request = request.header(name, val);
            }
        }

        let response = request.send().await.context("Failed to send request")?;
        let status = response.status();
        
        let headers_vec: Vec<(String, String)> = response.headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
        let body = response.bytes().await.context("Failed to read response")?;
        let charset = detect_charset(&headers_vec, &body);
        let text = decode_body(&body, &charset);

        Ok(Response {
            url: url.to_string(),
            status: status.as_u16(),
            headers: headers_vec,
            body: text,
            raw_bytes: body.to_vec(),
        })
    }

    pub async fn download(&self, url: &str) -> Result<Vec<u8>> {
        let response = self.client
            .get(url)
            .send()
            .await
            .context("Failed to download file")?;

        Ok(response.bytes().await?.to_vec())
    }
}

fn detect_charset(headers: &[(String, String)], body: &[u8]) -> String {
    for (key, value) in headers {
        if key.to_lowercase() == "content-type" {
            if value.contains("charset=") {
                return value
                    .split("charset=")
                    .nth(1)
                    .unwrap_or("utf-8")
                    .trim()
                    .to_string();
            }
        }
    }

    if body.len() > 4 {
        if body.starts_with(&[0xEF, 0xBB, 0xBF]) {
            return "utf-8".to_string();
        }
        if body.starts_with(&[0xFF, 0xFE]) {
            return "utf-16le".to_string();
        }
        if body.starts_with(&[0xFE, 0xFF]) {
            return "utf-16be".to_string();
        }
    }

    "utf-8".to_string()
}

fn decode_body(body: &[u8], charset: &str) -> String {
    match charset.to_lowercase().as_str() {
        "utf-8" | "utf8" => String::from_utf8_lossy(body).to_string(),
        "iso-8859-1" | "latin1" => {
            let mut s = String::with_capacity(body.len());
            for &b in body {
                s.push(b as char);
            }
            s
        }
        "windows-1252" => {
            let mut s = String::with_capacity(body.len());
            for &b in body {
                s.push(b as char);
            }
            s
        }
        _ => String::from_utf8_lossy(body).to_string(),
    }
}

#[derive(Debug, Clone)]
pub struct Response {
    pub url: String,
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
    pub raw_bytes: Vec<u8>,
}

impl Default for Fetcher {
    fn default() -> Self {
        Self::new().expect("Failed to create default Fetcher")
    }
}

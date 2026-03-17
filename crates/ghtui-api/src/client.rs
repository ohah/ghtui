use std::sync::Arc;
use std::sync::Mutex;

use lru::LruCache;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use reqwest::Client;

use crate::error::ApiError;
use crate::rate_limit::RateLimitState;

#[derive(Clone)]
pub struct GithubClient {
    pub(crate) http: Client,
    pub(crate) base_url: String,
    pub(crate) token: String,
    pub(crate) cache: Arc<Mutex<LruCache<String, CachedResponse>>>,
    pub(crate) rate_limit: Arc<Mutex<RateLimitState>>,
}

#[derive(Debug, Clone)]
pub struct CachedResponse {
    pub body: String,
    pub etag: Option<String>,
    pub cached_at: std::time::Instant,
    pub ttl_secs: u64,
}

impl CachedResponse {
    pub fn is_expired(&self) -> bool {
        self.cached_at.elapsed().as_secs() > self.ttl_secs
    }
}

impl GithubClient {
    pub fn new(token: String) -> Result<Self, ApiError> {
        Self::with_base_url(token, "https://api.github.com".to_string())
    }

    pub fn with_base_url(token: String, base_url: String) -> Result<Self, ApiError> {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token))
                .map_err(|e| ApiError::Other(e.to_string()))?,
        );
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/vnd.github+json"),
        );
        headers.insert(USER_AGENT, HeaderValue::from_static("ghtui"));
        headers.insert(
            "X-GitHub-Api-Version",
            HeaderValue::from_static("2022-11-28"),
        );

        let http = Client::builder()
            .default_headers(headers)
            .build()
            .map_err(ApiError::Http)?;

        let cache = Arc::new(Mutex::new(LruCache::new(
            std::num::NonZeroUsize::new(500).unwrap(),
        )));

        Ok(Self {
            http,
            base_url,
            token,
            cache,
            rate_limit: Arc::new(Mutex::new(RateLimitState::default())),
        })
    }

    pub fn rate_limit_info(&self) -> RateLimitState {
        self.rate_limit.lock().unwrap().clone()
    }

    pub(crate) fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    pub(crate) async fn get(&self, path: &str) -> Result<String, ApiError> {
        self.get_with_ttl(path, 30).await
    }

    pub(crate) async fn get_with_ttl(&self, path: &str, ttl_secs: u64) -> Result<String, ApiError> {
        let url = self.url(path);
        let cache_key = url.clone();

        // Check cache
        {
            let cache = self.cache.lock().unwrap();
            if let Some(cached) = cache.peek(&cache_key) {
                if !cached.is_expired() {
                    return Ok(cached.body.clone());
                }
            }
        }

        let response = self.http.get(&url).send().await?;

        self.update_rate_limit(&response);

        let status = response.status();
        let etag = response
            .headers()
            .get("etag")
            .and_then(|v| v.to_str().ok())
            .map(String::from);

        let body = response.text().await?;

        if !status.is_success() {
            return Err(self.parse_error(status.as_u16(), &body));
        }

        // Store in cache
        {
            let mut cache = self.cache.lock().unwrap();
            cache.put(
                cache_key,
                CachedResponse {
                    body: body.clone(),
                    etag,
                    cached_at: std::time::Instant::now(),
                    ttl_secs,
                },
            );
        }

        Ok(body)
    }

    pub(crate) async fn get_raw(&self, url: &str) -> Result<String, ApiError> {
        let response = self.http.get(url).send().await?;
        self.update_rate_limit(&response);

        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            return Err(self.parse_error(status.as_u16(), &body));
        }

        Ok(body)
    }

    pub(crate) async fn post(&self, path: &str, body: &serde_json::Value) -> Result<String, ApiError> {
        let url = self.url(path);
        let response = self.http.post(&url).json(body).send().await?;

        self.update_rate_limit(&response);

        let status = response.status();
        let text = response.text().await?;

        if !status.is_success() {
            return Err(self.parse_error(status.as_u16(), &text));
        }

        Ok(text)
    }

    pub(crate) async fn patch(&self, path: &str, body: &serde_json::Value) -> Result<String, ApiError> {
        let url = self.url(path);
        let response = self.http.patch(&url).json(body).send().await?;

        self.update_rate_limit(&response);

        let status = response.status();
        let text = response.text().await?;

        if !status.is_success() {
            return Err(self.parse_error(status.as_u16(), &text));
        }

        Ok(text)
    }

    pub(crate) async fn put(&self, path: &str, body: &serde_json::Value) -> Result<String, ApiError> {
        let url = self.url(path);
        let response = self.http.put(&url).json(body).send().await?;

        self.update_rate_limit(&response);

        let status = response.status();
        let text = response.text().await?;

        if !status.is_success() {
            return Err(self.parse_error(status.as_u16(), &text));
        }

        Ok(text)
    }

    pub(crate) async fn delete(&self, path: &str) -> Result<(), ApiError> {
        let url = self.url(path);
        let response = self.http.delete(&url).send().await?;

        self.update_rate_limit(&response);

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await?;
            return Err(self.parse_error(status.as_u16(), &text));
        }

        Ok(())
    }

    fn update_rate_limit(&self, response: &reqwest::Response) {
        let headers = response.headers();

        let remaining = headers
            .get("x-ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u32>().ok());

        let limit = headers
            .get("x-ratelimit-limit")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u32>().ok());

        let reset = headers
            .get("x-ratelimit-reset")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<i64>().ok());

        if let (Some(remaining), Some(limit), Some(reset)) = (remaining, limit, reset) {
            let mut rl = self.rate_limit.lock().unwrap();
            rl.remaining = remaining;
            rl.limit = limit;
            rl.reset_at = reset;
        }
    }

    fn parse_error(&self, status: u16, body: &str) -> ApiError {
        match status {
            401 => ApiError::Unauthorized,
            403 => {
                if body.contains("rate limit") {
                    let rl = self.rate_limit.lock().unwrap();
                    ApiError::RateLimit {
                        reset_at: rl.reset_at,
                        remaining: rl.remaining,
                    }
                } else {
                    ApiError::GitHub {
                        status,
                        message: extract_message(body),
                    }
                }
            }
            404 => ApiError::NotFound(extract_message(body)),
            _ => ApiError::GitHub {
                status,
                message: extract_message(body),
            },
        }
    }
}

fn extract_message(body: &str) -> String {
    serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|v| v.get("message").and_then(|m| m.as_str()).map(String::from))
        .unwrap_or_else(|| body.to_string())
}

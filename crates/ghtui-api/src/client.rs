use std::sync::Arc;
use std::sync::Mutex;

use lru::LruCache;
use reqwest::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT};

use crate::disk_cache::DiskCache;
use crate::error::ApiError;
use crate::rate_limit::RateLimitState;

#[derive(Clone)]
#[allow(dead_code)]
pub struct GithubClient {
    pub(crate) http: Client,
    pub(crate) base_url: String,
    pub(crate) token: String,
    pub(crate) cache: Arc<Mutex<LruCache<String, CachedResponse>>>,
    pub(crate) rate_limit: Arc<Mutex<RateLimitState>>,
    pub(crate) disk_cache: Option<DiskCache>,
    /// True when the last response was served from disk cache (stale data).
    pub serving_cached: Arc<std::sync::atomic::AtomicBool>,
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

        let disk_cache = DiskCache::new();

        Ok(Self {
            http,
            base_url,
            token,
            cache,
            rate_limit: Arc::new(Mutex::new(RateLimitState::default())),
            disk_cache,
            serving_cached: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        })
    }

    pub fn rate_limit_info(&self) -> RateLimitState {
        self.rate_limit.lock().unwrap().clone()
    }

    /// Run disk cache cleanup, removing entries older than 24 hours.
    pub fn cleanup_disk_cache(&self) {
        if let Some(ref dc) = self.disk_cache {
            dc.cleanup();
        }
    }

    pub(crate) fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    /// Execute a GraphQL query/mutation.
    pub(crate) async fn graphql(
        &self,
        query: &str,
        variables: serde_json::Value,
    ) -> Result<serde_json::Value, ApiError> {
        let body = serde_json::json!({
            "query": query,
            "variables": variables,
        });
        let response_text = self.post("/graphql", &body).await?;
        let result: serde_json::Value = serde_json::from_str(&response_text)?;

        // Check for GraphQL errors
        if let Some(errors) = result.get("errors")
            && let Some(first) = errors.as_array().and_then(|a| a.first())
        {
            let message = first
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("GraphQL error");
            return Err(ApiError::Other(message.to_string()));
        }

        Ok(result)
    }

    pub(crate) async fn get(&self, path: &str) -> Result<String, ApiError> {
        self.get_with_ttl(path, 30).await
    }

    pub(crate) async fn get_with_ttl(&self, path: &str, ttl_secs: u64) -> Result<String, ApiError> {
        let url = self.url(path);
        let cache_key = url.clone();

        // Check in-memory LRU cache
        {
            let cache = self.cache.lock().unwrap();
            if let Some(cached) = cache.peek(&cache_key)
                && !cached.is_expired()
            {
                return Ok(cached.body.clone());
            }
        }

        // Try HTTP request; on failure, fall back to disk cache
        let http_result = self.http.get(&url).send().await;

        match http_result {
            Ok(response) => {
                self.update_rate_limit(&response);

                let status = response.status();
                let etag = response
                    .headers()
                    .get("etag")
                    .and_then(|v| v.to_str().ok())
                    .map(String::from);

                let body = response.text().await?;

                if !status.is_success() {
                    let error = self.parse_error(status.as_u16(), &body);

                    // On rate limit or server error, try disk cache fallback
                    if (matches!(error, ApiError::RateLimit { .. })
                        || matches!(error, ApiError::GitHub { status, .. } if status >= 500))
                        && let Some(cached_body) = self.disk_cache_get(&cache_key)
                    {
                        tracing::warn!("API error, serving disk-cached response for {}", path);
                        return Ok(cached_body);
                    }

                    return Err(error);
                }

                // Store in LRU cache
                {
                    let mut cache = self.cache.lock().unwrap();
                    cache.put(
                        cache_key.clone(),
                        CachedResponse {
                            body: body.clone(),
                            etag,
                            cached_at: std::time::Instant::now(),
                            ttl_secs,
                        },
                    );
                }

                // Store in disk cache
                if let Some(ref dc) = self.disk_cache {
                    dc.set(&cache_key, &body);
                }

                // Clear stale flag — fresh data
                self.serving_cached
                    .store(false, std::sync::atomic::Ordering::Relaxed);

                Ok(body)
            }
            Err(e) => {
                // Network error — try disk cache fallback
                if let Some(cached_body) = self.disk_cache_get(&cache_key) {
                    tracing::warn!(
                        "Network error, serving disk-cached response for {}: {}",
                        path,
                        e
                    );
                    return Ok(cached_body);
                }
                Err(ApiError::Http(e))
            }
        }
    }

    /// Read from disk cache if available. Sets serving_cached flag.
    fn disk_cache_get(&self, url: &str) -> Option<String> {
        let result = self.disk_cache.as_ref()?.get(url);
        if result.is_some() {
            self.serving_cached
                .store(true, std::sync::atomic::Ordering::Relaxed);
        }
        result
    }

    /// Check and reset the stale-data flag.
    pub fn take_serving_cached(&self) -> bool {
        self.serving_cached
            .swap(false, std::sync::atomic::Ordering::Relaxed)
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

    pub(crate) async fn post(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> Result<String, ApiError> {
        let url = self.url(path);
        let response = self.http.post(&url).json(body).send().await?;

        self.update_rate_limit(&response);
        self.invalidate_cache(path);

        let status = response.status();
        let text = response.text().await?;

        if !status.is_success() {
            return Err(self.parse_error(status.as_u16(), &text));
        }

        Ok(text)
    }

    pub(crate) async fn patch(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> Result<String, ApiError> {
        let url = self.url(path);
        let response = self.http.patch(&url).json(body).send().await?;

        self.update_rate_limit(&response);
        self.invalidate_cache(path);

        let status = response.status();
        let text = response.text().await?;

        if !status.is_success() {
            return Err(self.parse_error(status.as_u16(), &text));
        }

        Ok(text)
    }

    pub(crate) async fn put(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> Result<String, ApiError> {
        let url = self.url(path);
        let response = self.http.put(&url).json(body).send().await?;

        self.update_rate_limit(&response);
        self.invalidate_cache(path);

        let status = response.status();
        let text = response.text().await?;

        if !status.is_success() {
            return Err(self.parse_error(status.as_u16(), &text));
        }

        Ok(text)
    }

    #[allow(dead_code)]
    pub(crate) async fn delete(&self, path: &str) -> Result<(), ApiError> {
        let url = self.url(path);
        let response = self.http.delete(&url).send().await?;

        self.update_rate_limit(&response);
        self.invalidate_cache(path);

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await?;
            return Err(self.parse_error(status.as_u16(), &text));
        }

        Ok(())
    }

    /// Invalidate cache entries that match or are prefixed by the given path.
    /// E.g. PATCH /repos/o/r/issues/1 invalidates both the issue and its parent list.
    fn invalidate_cache(&self, path: &str) {
        let url = self.url(path);
        let mut cache = self.cache.lock().unwrap();

        // Collect keys to remove (exact match + parent paths)
        let keys_to_remove: Vec<String> = cache
            .iter()
            .map(|(k, _)| k.clone())
            .filter(|k| {
                // Remove exact match
                k == &url
                // Remove parent paths (e.g. /issues when /issues/1 is modified)
                || url.starts_with(k.as_str())
                // Remove child paths (e.g. /issues/1/comments when /issues/1 is modified)
                || k.starts_with(url.as_str())
            })
            .collect();

        for key in keys_to_remove {
            cache.pop(&key);
        }
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

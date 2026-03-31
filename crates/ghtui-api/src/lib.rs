pub mod client;
pub mod diff;
pub mod disk_cache;
pub mod endpoints;
pub mod error;
pub mod pagination;
pub mod rate_limit;

pub use client::GithubClient;
pub use disk_cache::DiskCache;
pub use error::ApiError;
pub use rate_limit::RateLimitState;
// Test: multi-file diff

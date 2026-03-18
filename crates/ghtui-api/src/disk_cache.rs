use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

#[derive(Clone)]
pub struct DiskCache {
    dir: PathBuf,
}

impl DiskCache {
    pub fn new() -> Option<Self> {
        let dir = dirs::cache_dir()?.join("ghtui").join("api");
        std::fs::create_dir_all(&dir).ok()?;
        Some(Self { dir })
    }

    fn key_path(&self, url: &str) -> PathBuf {
        let mut hasher = DefaultHasher::new();
        url.hash(&mut hasher);
        let hash = hasher.finish();
        self.dir.join(format!("{:x}.json", hash))
    }

    pub fn get(&self, url: &str) -> Option<String> {
        let path = self.key_path(url);
        std::fs::read_to_string(&path).ok()
    }

    pub fn set(&self, url: &str, body: &str) {
        let path = self.key_path(url);
        let _ = std::fs::write(&path, body);
    }

    /// Remove expired entries (older than 24 hours).
    pub fn cleanup(&self) {
        if let Ok(entries) = std::fs::read_dir(&self.dir) {
            let cutoff =
                std::time::SystemTime::now() - std::time::Duration::from_secs(24 * 60 * 60);
            for entry in entries.flatten() {
                if let Ok(meta) = entry.metadata() {
                    if let Ok(modified) = meta.modified() {
                        if modified < cutoff {
                            let _ = std::fs::remove_file(entry.path());
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disk_cache_roundtrip() {
        let dir = std::env::temp_dir().join("ghtui_test_cache");
        let _ = std::fs::create_dir_all(&dir);
        let cache = DiskCache { dir: dir.clone() };

        cache.set("https://api.github.com/repos/test", "{\"id\": 1}");
        let result = cache.get("https://api.github.com/repos/test");
        assert_eq!(result, Some("{\"id\": 1}".to_string()));

        // Different URL returns None
        assert!(cache.get("https://api.github.com/repos/other").is_none());

        // Cleanup
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_disk_cache_miss() {
        let dir = std::env::temp_dir().join("ghtui_test_cache_miss");
        let _ = std::fs::create_dir_all(&dir);
        let cache = DiskCache { dir: dir.clone() };

        assert!(cache.get("https://nonexistent").is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }
}

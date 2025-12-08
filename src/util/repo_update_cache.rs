use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Build a cache file path scoped to a specific config file path. Different
/// projects (different `vesshelm.yaml` locations) get distinct cache files,
/// while repeated runs against the same config share one.
pub fn cache_path_for(config_path: &Path) -> PathBuf {
    let canonical = fs::canonicalize(config_path).unwrap_or_else(|_| config_path.to_path_buf());
    let mut hasher = DefaultHasher::new();
    canonical.hash(&mut hasher);
    let key = hasher.finish();
    std::env::temp_dir().join(format!("vesshelm-repo-update-{:016x}.timestamp", key))
}

/// Returns true if `helm repo update` should run.
/// `cache_minutes == 0` disables the cache (always returns true).
pub fn should_update(cache_path: &Path, cache_minutes: u64) -> bool {
    if cache_minutes == 0 {
        return true;
    }
    let Ok(content) = fs::read_to_string(cache_path) else {
        return true;
    };
    let Ok(last) = content.trim().parse::<u64>() else {
        return true;
    };
    let Ok(now) = SystemTime::now().duration_since(UNIX_EPOCH) else {
        return true;
    };
    now.as_secs().saturating_sub(last) >= cache_minutes * 60
}

pub fn last_update_age_minutes(cache_path: &Path) -> Option<u64> {
    let content = fs::read_to_string(cache_path).ok()?;
    let last: u64 = content.trim().parse().ok()?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
    Some(now.saturating_sub(last) / 60)
}

pub fn mark_updated(cache_path: &Path) {
    if let Ok(now) = SystemTime::now().duration_since(UNIX_EPOCH) {
        let _ = fs::write(cache_path, now.as_secs().to_string());
    }
}

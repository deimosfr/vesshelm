use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use vesshelm::util::repo_update_cache::{
    cache_path_for, last_update_age_minutes, mark_updated, should_update,
};

fn unique_cache_file(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "vesshelm-test-{}-{}-{}",
        name,
        std::process::id(),
        // nanoseconds add per-test uniqueness even within the same process
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0)
    ));
    let _ = fs::remove_file(&path);
    path
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[test]
fn should_update_when_cache_file_missing() {
    let p = unique_cache_file("missing");
    assert!(should_update(&p, 60));
}

#[test]
fn should_update_always_when_cache_minutes_zero() {
    let p = unique_cache_file("zero");
    fs::write(&p, now_secs().to_string()).unwrap();
    assert!(should_update(&p, 0));
    let _ = fs::remove_file(&p);
}

#[test]
fn should_not_update_when_recent() {
    let p = unique_cache_file("recent");
    fs::write(&p, now_secs().to_string()).unwrap();
    assert!(!should_update(&p, 60));
    let _ = fs::remove_file(&p);
}

#[test]
fn should_update_when_cache_expired() {
    let p = unique_cache_file("expired");
    let stale = now_secs() - 7200;
    fs::write(&p, stale.to_string()).unwrap();
    assert!(should_update(&p, 60));
    let _ = fs::remove_file(&p);
}

#[test]
fn should_update_when_cache_corrupt() {
    let p = unique_cache_file("corrupt");
    fs::write(&p, "not-a-number").unwrap();
    assert!(should_update(&p, 60));
    let _ = fs::remove_file(&p);
}

#[test]
fn mark_updated_writes_recent_timestamp() {
    let p = unique_cache_file("mark");
    mark_updated(&p);
    let content = fs::read_to_string(&p).unwrap();
    let stored: u64 = content.trim().parse().unwrap();
    let delta = now_secs().saturating_sub(stored);
    assert!(delta < 5, "expected fresh timestamp, delta={}", delta);
    assert!(!should_update(&p, 60));
    let _ = fs::remove_file(&p);
}

#[test]
fn last_update_age_minutes_reports_age() {
    let p = unique_cache_file("age");
    let past = now_secs() - 600; // 10 min
    fs::write(&p, past.to_string()).unwrap();
    let age = last_update_age_minutes(&p).unwrap();
    assert!((9..=11).contains(&age), "age was {}", age);
    let _ = fs::remove_file(&p);
}

#[test]
fn cache_path_for_is_stable_per_config_and_distinct_across_configs() {
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a.yaml");
    let b = dir.path().join("b.yaml");
    fs::write(&a, "x").unwrap();
    fs::write(&b, "y").unwrap();

    let a1 = cache_path_for(&a);
    let a2 = cache_path_for(&a);
    let b1 = cache_path_for(&b);

    assert_eq!(a1, a2, "same config path must yield same cache path");
    assert_ne!(
        a1, b1,
        "different config paths must yield distinct cache paths"
    );
}

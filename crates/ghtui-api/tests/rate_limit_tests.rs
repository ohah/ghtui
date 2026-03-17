use ghtui_api::RateLimitState;

#[test]
fn test_rate_limit_default() {
    let rl = RateLimitState::default();
    assert_eq!(rl.limit, 0);
    assert_eq!(rl.remaining, 0);
    assert!(!rl.is_exhausted()); // limit is 0, so not "exhausted"
    assert!(!rl.is_low());
}

#[test]
fn test_rate_limit_healthy() {
    let rl = RateLimitState {
        limit: 5000,
        remaining: 4500,
        reset_at: 0,
    };
    assert!(!rl.is_exhausted());
    assert!(!rl.is_low());
    assert!((rl.usage_pct() - 10.0).abs() < 0.1);
}

#[test]
fn test_rate_limit_low() {
    let rl = RateLimitState {
        limit: 5000,
        remaining: 50,
        reset_at: 0,
    };
    assert!(!rl.is_exhausted());
    assert!(rl.is_low());
}

#[test]
fn test_rate_limit_exhausted() {
    let rl = RateLimitState {
        limit: 5000,
        remaining: 0,
        reset_at: 1234567890,
    };
    assert!(rl.is_exhausted());
    assert!(rl.is_low());
    assert!((rl.usage_pct() - 100.0).abs() < 0.1);
}

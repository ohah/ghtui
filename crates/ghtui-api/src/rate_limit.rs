#[derive(Debug, Clone, Default)]
pub struct RateLimitState {
    pub limit: u32,
    pub remaining: u32,
    pub reset_at: i64,
}

impl RateLimitState {
    pub fn is_exhausted(&self) -> bool {
        self.remaining == 0 && self.limit > 0
    }

    pub fn is_low(&self) -> bool {
        self.remaining < 100 && self.limit > 0
    }

    pub fn usage_pct(&self) -> f32 {
        if self.limit == 0 {
            return 0.0;
        }
        ((self.limit - self.remaining) as f32 / self.limit as f32) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let rl = RateLimitState::default();
        assert_eq!(rl.limit, 0);
        assert_eq!(rl.remaining, 0);
        assert_eq!(rl.reset_at, 0);
    }

    #[test]
    fn test_zero_limit_is_neither_exhausted_nor_low() {
        let rl = RateLimitState {
            limit: 0,
            remaining: 0,
            reset_at: 0,
        };
        assert!(!rl.is_exhausted());
        assert!(!rl.is_low());
    }

    #[test]
    fn test_low_boundary_at_100() {
        // remaining == 100 should NOT be low
        let rl = RateLimitState {
            limit: 5000,
            remaining: 100,
            reset_at: 0,
        };
        assert!(!rl.is_low());

        // remaining == 99 should be low
        let rl = RateLimitState {
            limit: 5000,
            remaining: 99,
            reset_at: 0,
        };
        assert!(rl.is_low());
    }

    #[test]
    fn test_usage_pct_zero_limit() {
        let rl = RateLimitState {
            limit: 0,
            remaining: 0,
            reset_at: 0,
        };
        assert_eq!(rl.usage_pct(), 0.0);
    }

    #[test]
    fn test_usage_pct_half() {
        let rl = RateLimitState {
            limit: 1000,
            remaining: 500,
            reset_at: 0,
        };
        assert!((rl.usage_pct() - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_usage_pct_full_remaining() {
        let rl = RateLimitState {
            limit: 5000,
            remaining: 5000,
            reset_at: 0,
        };
        assert_eq!(rl.usage_pct(), 0.0);
    }

    #[test]
    fn test_exhausted_and_low_together() {
        let rl = RateLimitState {
            limit: 5000,
            remaining: 0,
            reset_at: 0,
        };
        assert!(rl.is_exhausted());
        assert!(rl.is_low());
    }

    #[test]
    fn test_not_exhausted_with_remaining() {
        let rl = RateLimitState {
            limit: 5000,
            remaining: 1,
            reset_at: 0,
        };
        assert!(!rl.is_exhausted());
    }
}

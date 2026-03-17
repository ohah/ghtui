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

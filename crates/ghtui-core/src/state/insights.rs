use crate::types::insights::{CommitActivity, ContributorStats, TrafficClones, TrafficViews};

#[derive(Debug, Default)]
pub struct InsightsState {
    pub contributors: Vec<ContributorStats>,
    pub commit_activity: Vec<CommitActivity>,
    pub traffic_clones: Option<TrafficClones>,
    pub traffic_views: Option<TrafficViews>,
    pub tab: usize, // 0=Contributors, 1=Commit Activity, 2=Traffic
    pub scroll: usize,
}

impl InsightsState {
    pub fn tab_count(&self) -> usize {
        3
    }
}

use crate::types::insights::{
    CodeFrequency, CommitActivity, ContributorStats, Fork, TrafficClones, TrafficViews,
};

#[derive(Debug, Default)]
pub struct InsightsState {
    pub contributors: Vec<ContributorStats>,
    pub commit_activity: Vec<CommitActivity>,
    pub traffic_clones: Option<TrafficClones>,
    pub traffic_views: Option<TrafficViews>,
    pub code_frequency: Vec<CodeFrequency>,
    pub forks: Vec<Fork>,
    pub tab: usize, // 0=Contributors, 1=Commit Activity, 2=Traffic, 3=Code Frequency, 4=Forks
    pub scroll: usize,
}

impl InsightsState {
    pub fn tab_count(&self) -> usize {
        5
    }
}

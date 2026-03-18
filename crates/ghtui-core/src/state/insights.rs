use crate::types::insights::{
    CodeFrequency, CommitActivity, ContributorStats, DependencyEntry, Fork, TrafficClones,
    TrafficViews,
};

#[derive(Debug, Default)]
pub struct InsightsState {
    pub contributors: Vec<ContributorStats>,
    pub commit_activity: Vec<CommitActivity>,
    pub traffic_clones: Option<TrafficClones>,
    pub traffic_views: Option<TrafficViews>,
    pub code_frequency: Vec<CodeFrequency>,
    pub forks: Vec<Fork>,
    pub dependencies: Vec<DependencyEntry>,
    pub tab: usize, // 0=Contributors, 1=Commit Activity, 2=Traffic, 3=Code Frequency, 4=Forks, 5=Dependencies
    pub scroll: usize,
}

impl InsightsState {
    pub fn tab_count(&self) -> usize {
        6
    }
}

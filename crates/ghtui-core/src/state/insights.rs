use crate::types::insights::{
    CodeFrequency, CommitActivity, ContributorStats, DependencyEntry, Fork, TrafficClones,
    TrafficViews,
};

#[derive(Debug)]
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
    pub sidebar_focused: bool,
}

impl Default for InsightsState {
    fn default() -> Self {
        Self {
            contributors: Vec::new(),
            commit_activity: Vec::new(),
            traffic_clones: None,
            traffic_views: None,
            code_frequency: Vec::new(),
            forks: Vec::new(),
            dependencies: Vec::new(),
            tab: 0,
            scroll: 0,
            sidebar_focused: true,
        }
    }
}

impl InsightsState {
    pub fn tab_count(&self) -> usize {
        6
    }
}

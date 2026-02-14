use serde::{Deserialize, Serialize};

use crate::models::{area::Area, project::Project, task::Task};

/// Current schema version
pub const CURRENT_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
pub struct Store {
    pub version: u32,
    pub tasks: Vec<Task>,
    pub projects: Vec<Project>,
    pub areas: Vec<Area>,
}

impl Default for Store {
    fn default() -> Self {
        Self {
            version: CURRENT_VERSION,
            tasks: vec![],
            projects: vec![],
            areas: vec![],
        }
    }
}

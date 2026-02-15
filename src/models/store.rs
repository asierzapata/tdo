use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::models::{area::Area, project::Project, task::Task};

/// Current schema version
pub const CURRENT_VERSION: u32 = 2;

/// Storage representation (how data lives on disk as JSON)
#[derive(Serialize, Deserialize)]
pub struct StoredStore {
    pub version: u32,
    pub next_task_number: u64,
    pub tasks: Vec<Task>,
    pub projects: Vec<Project>,
    pub areas: Vec<Area>,
}

impl Default for StoredStore {
    fn default() -> Self {
        Self {
            version: CURRENT_VERSION,
            next_task_number: 1,
            tasks: vec![],
            projects: vec![],
            areas: vec![],
        }
    }
}

/// In-memory representation (how we work with data in the app)
pub struct Store {
    pub version: u32,
    pub next_task_number: u64,
    pub tasks: HashMap<Uuid, Task>,
    pub projects: HashMap<Uuid, Project>,
    pub areas: HashMap<Uuid, Area>,
}

impl Default for Store {
    fn default() -> Self {
        Self {
            version: CURRENT_VERSION,
            next_task_number: 1,
            tasks: HashMap::new(),
            projects: HashMap::new(),
            areas: HashMap::new(),
        }
    }
}

impl Store {
    /// Convert from storage format (Vec) to working format (HashMap)
    pub fn from_stored(stored: StoredStore) -> Self {
        let tasks: HashMap<_, _> = stored.tasks.into_iter().map(|t| (t.id, t)).collect();

        let projects: HashMap<_, _> = stored.projects.into_iter().map(|p| (p.id, p)).collect();

        let areas: HashMap<_, _> = stored.areas.into_iter().map(|a| (a.id, a)).collect();

        Self {
            version: stored.version,
            next_task_number: stored.next_task_number,
            tasks,
            projects,
            areas,
        }
    }

    /// Convert from working format (HashMap) to storage format (Vec)
    pub fn to_stored(&self) -> StoredStore {
        StoredStore {
            version: self.version,
            next_task_number: self.next_task_number,
            tasks: self.tasks.values().cloned().collect(),
            projects: self.projects.values().cloned().collect(),
            areas: self.areas.values().cloned().collect(),
        }
    }

    /// Add a task to the store, assigning it the next task_number
    pub fn add_task(&mut self, mut task: Task) {
        task.task_number = self.next_task_number;
        self.next_task_number += 1;
        self.tasks.insert(task.id, task);
    }

    /// Add a project to the store
    pub fn add_project(&mut self, project: Project) {
        self.projects.insert(project.id, project);
    }

    /// Add an area to the store
    pub fn add_area(&mut self, area: Area) {
        self.areas.insert(area.id, area);
    }

    /// Get a task by ID
    pub fn get_task(&self, id: Uuid) -> Option<&Task> {
        self.tasks.get(&id)
    }

    /// Look up a task by its user-facing task_number
    pub fn get_task_by_number(&self, number: u64) -> Option<&Task> {
        self.tasks.values().find(|t| t.task_number == number)
    }

    /// Get a project by ID
    pub fn get_project(&self, id: Uuid) -> Option<&Project> {
        self.projects.get(&id)
    }

    /// Get an area by ID
    pub fn get_area(&self, id: Uuid) -> Option<&Area> {
        self.areas.get(&id)
    }
}

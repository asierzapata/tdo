use jiff::civil::Date;
use thiserror::Error;
use uuid::Uuid;

use crate::{
    models::{
        store::Store,
        task::{Task, When},
    },
    storage::{Storage, StorageError},
};

#[derive(Debug, Error)]
pub enum AddTaskError {
    #[error("Project '{0}' not found")]
    ProjectNotFound(String),

    #[error("Project name is ambiguous. Multiple projects found: {}", .0.join(", "))]
    AmbiguousProjectName(Vec<String>),

    #[error("Area '{0}' not found")]
    AreaNotFound(String),

    #[error("Area name is ambiguous. Multiple areas found: {}", .0.join(", "))]
    AmbiguousAreaName(Vec<String>),

    #[error("Invalid deadline date '{0}': {1}")]
    InvalidDeadline(String, String),

    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
}

pub struct AddTaskParameters {
    pub title: String,
    pub notes: Option<String>,
    pub when: When,
    pub deadline: Option<String>,
    pub project: Option<String>,
    pub area: Option<String>,
    pub tags: Vec<String>,
}

pub fn add_task(
    store: &mut Store,
    storage: &impl Storage,
    parameters: AddTaskParameters,
) -> Result<Task, AddTaskError> {
    // 1. Validate and resolve project name to project ID
    let project_id = if let Some(project_name) = parameters.project {
        let matching_projects: Vec<_> = store
            .projects
            .values()
            .filter(|p| p.name.to_lowercase().contains(&project_name.to_lowercase()))
            .collect();

        match matching_projects.len() {
            0 => return Err(AddTaskError::ProjectNotFound(project_name)),
            1 => Some(matching_projects[0].id),
            _ => {
                let names: Vec<String> = matching_projects.iter().map(|p| p.name.clone()).collect();
                return Err(AddTaskError::AmbiguousProjectName(names));
            }
        }
    } else {
        None
    };

    // 2. Validate and resolve area name to area ID
    let area_id = if let Some(area_name) = parameters.area {
        let matching_areas: Vec<_> = store
            .areas
            .values()
            .filter(|a| a.name.to_lowercase().contains(&area_name.to_lowercase()))
            .collect();

        match matching_areas.len() {
            0 => return Err(AddTaskError::AreaNotFound(area_name)),
            1 => Some(matching_areas[0].id),
            _ => {
                let names: Vec<String> = matching_areas.iter().map(|a| a.name.clone()).collect();
                return Err(AddTaskError::AmbiguousAreaName(names));
            }
        }
    } else {
        None
    };

    // 3. Parse deadline if provided
    let deadline = if let Some(deadline_str) = parameters.deadline {
        Some(
            deadline_str
                .parse::<Date>()
                .map_err(|e| AddTaskError::InvalidDeadline(deadline_str.clone(), e.to_string()))?,
        )
    } else {
        None
    };

    // 4. Create the task (task_number will be assigned by store.add_task)
    let task = Task {
        id: Uuid::new_v4(),
        task_number: 0,
        title: parameters.title,
        notes: parameters.notes,
        project_id,
        area_id,
        tags: parameters.tags,
        when: parameters.when,
        deadline,
        defer_until: None,
        checklist: vec![],
        completed_at: None,
        deleted_at: None,
        created_at: jiff::Timestamp::now(),
    };

    let task_id = task.id;

    // 5. Add to store (assigns task_number)
    store.add_task(task);

    // 6. Persist to storage
    storage.save(store)?;

    // 7. Return the created task (with the assigned task_number)
    Ok(store.get_task(task_id).unwrap().clone())
}

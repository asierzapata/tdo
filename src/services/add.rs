use crate::models::task::{Task, When};

pub struct AddTaskParameters {
    title: String,
    notes: Option<String>,
    when: When,
    deadline: Option<String>,
    project: Option<String>,
    tags: Vec<String>,
}

pub fn add_task(parameters: AddTaskParameters) -> Task {
    // 1. get if project exists with that name
    // 1.a if not return error with similar names?
    // 2. mount the whole task
    // 3. persist it
    // 4. return the task
    todo!()
}

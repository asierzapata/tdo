// src/model.rs

use jiff::Timestamp;
use jiff::civil::Date;
use uuid::Uuid;

pub struct Task {
    /// UUID to identify the task
    pub id: Uuid,
    /// Title of the task
    pub title: String,
    /// Notes of the task
    pub notes: Option<String>,
    /// The project of this task if it belongs to any
    pub project_id: Option<Uuid>,
    /// Tags of the task
    pub tags: Vec<String>,
    /// When the user wants do to this task
    pub when: When,
    /// Deadline for this task
    pub deadline: Option<Date>,
    /// Defered date when to surface again the task
    pub defer_until: Option<Date>,
    /// Sub tasks of the main task - Modeled as a lighter task called ChecklistItem
    pub checklist: Vec<ChecklistItem>,
    /// When the task was completed
    pub completed_at: Option<Timestamp>,
    /// When the task was deleted
    pub deleted_at: Option<Timestamp>,
    /// When the task was created
    pub created_at: Timestamp,
}

pub enum When {
    Inbox,
    Today { evening: bool },
    Someday,
    Anytime,
    Scheduled(Date),
}

pub enum WhenInstantiationError {
    ScheduleAtIncorrect,
}

impl When {
    pub fn create_when_from_command_flags(
        today: bool,
        evening: bool,
        someday: bool,
        anytime: bool,
        schedule_at: Option<String>,
    ) -> Result<When, WhenInstantiationError> {
        if today {
            Ok(When::Today { evening })
        } else if someday {
            Ok(When::Someday)
        } else if anytime {
            Ok(When::Anytime)
        } else if let Some(string_date) = schedule_at {
            string_date
                .parse()
                .map(When::Scheduled)
                .map_err(|_| WhenInstantiationError::ScheduleAtIncorrect)
        } else {
            Ok(When::Inbox)
        }
    }
}

pub struct ChecklistItem {
    pub id: Uuid,
    pub title: String,
    pub completed: bool,
}

pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub area_id: Option<Uuid>,
    pub notes: Option<String>,
    pub deadline: Option<Date>,
    pub completed_at: Option<Timestamp>,
    pub created_at: Timestamp,
}

pub struct Area {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
}

pub struct Store {
    pub tasks: Vec<Task>,
    pub projects: Vec<Project>,
    pub areas: Vec<Area>,
}

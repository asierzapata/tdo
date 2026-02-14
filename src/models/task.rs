// src/model.rs

use jiff::Timestamp;
use jiff::civil::Date;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Task {
    /// UUID to identify the task
    pub id: Uuid,
    /// Title of the task
    pub title: String,
    /// Notes of the task
    pub notes: Option<String>,
    /// The project of this task if it belongs to any
    pub project_id: Option<Uuid>,
    /// The area of this task if it belongs to any (and no project)
    pub area_id: Option<Uuid>,
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

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(tag = "type")]
pub enum When {
    #[default]
    Inbox,
    Today {
        evening: bool,
    },
    Someday,
    Anytime,
    Scheduled(Date),
}

pub enum WhenInstantiationError {
    ScheduleAtIncorrect,
}

impl When {
    pub fn from_command_flags(
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

#[derive(Serialize, Deserialize, Clone)]
pub struct ChecklistItem {
    pub id: Uuid,
    pub title: String,
    pub completed: bool,
}

use jiff::Timestamp;
use jiff::civil::Date;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Default, Clone)]
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

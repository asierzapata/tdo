use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Default)]
pub struct Area {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
}

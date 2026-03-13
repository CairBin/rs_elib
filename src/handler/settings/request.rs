use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct UpdateSettingRequest {
    pub value: String,
}
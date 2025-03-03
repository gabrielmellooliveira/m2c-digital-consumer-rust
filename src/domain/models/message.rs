use lapin::types::Boolean;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use chrono::{Utc, DateTime};
use std::error::Error;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub identifier: String,
    pub message: String,
    pub phone_number: String,
    pub campaign_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted: Boolean
}

impl Message {
    pub fn build(data: Value) -> Result<Message, Box<dyn Error>> {
        let identifier = data["identifier"]
            .as_str()
            .ok_or("Missing 'identifier' field")?
            .to_string();

        let message = data["message"]
            .as_str()
            .ok_or("Missing 'message' field")?
            .to_string();

        let phone_number = data["phoneNumber"]
            .as_str()
            .ok_or("Missing 'phone_number' field")?
            .to_string();

        let campaign_id = data["campaignId"]
            .as_str()
            .ok_or("Missing 'campaign_id' field")?
            .to_string();

        let now = Utc::now(); 

        Ok(Message {
            identifier,
            message,
            phone_number,
            campaign_id,
            created_at: now,
            updated_at: now,
            deleted: false
        })
    }
}
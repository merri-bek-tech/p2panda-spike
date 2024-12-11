use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SiteRegistration {
    pub site_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SiteNotification {
    pub notification: String,
}

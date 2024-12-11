use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SiteRegistration {
    pub site_slug: String,
}

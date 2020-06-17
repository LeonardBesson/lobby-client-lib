use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    user_tag: String,
    display_name: String,
    avatar_url: Option<String>,
}

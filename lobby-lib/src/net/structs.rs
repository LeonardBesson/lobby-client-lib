use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    user_tag: String,
    display_name: String,
    avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FriendRequestAction {
    Accept,
    Decline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendRequest {
    id: String,
    state: String,
    user_profile: UserProfile,
}

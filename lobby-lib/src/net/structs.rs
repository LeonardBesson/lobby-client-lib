use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub user_tag: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FriendRequestActionChoice {
    Accept,
    Decline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendRequest {
    pub id: String,
    pub state: String,
    pub user_profile: UserProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Friend {
    pub user_profile: UserProfile,
    pub is_online: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LobbyInviteActionChoice {
    Accept,
    Decline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LobbyRole {
    Leader,
    Member,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LobbyMember {
    pub user_profile: UserProfile,
    pub role: LobbyRole,
    pub is_online: bool,
}

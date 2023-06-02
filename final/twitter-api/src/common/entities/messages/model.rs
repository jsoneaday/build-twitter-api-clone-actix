use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use sqlx::{FromRow};


#[derive(Deserialize, Serialize, FromRow, Clone, Debug)]
pub struct MessageQueryResult {
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub user_id: i64,
    pub body: Option<String>,
    pub image: Option<Vec<u8>>,
    pub likes: i32,
    pub msg_group_type: i32
}

#[derive(Deserialize, Serialize, FromRow, Clone, Debug)]
pub struct MessageWithProfileQueryResult {
    // messsage fields
    pub id: i64,
    pub updated_at: DateTime<Utc>,
    pub body: Option<String>,
    pub likes: i32,
    pub image: Option<Vec<u8>>,    
    pub msg_group_type: i32,
    // profile fields
    pub user_id: i64,
    pub user_name: String,
    pub full_name: String,
    pub avatar: Option<Vec<u8>>,
    // broadcast message fields
    pub broadcast_msg_id: Option<i64>    
}

#[derive(Deserialize, Serialize, FromRow, Clone, Debug)]
pub struct MessageWithFollowingAndBroadcastQueryResult {
    // messsage fields
    pub id: i64,
    pub updated_at: DateTime<Utc>,
    pub body: Option<String>,
    pub likes: i32,
    pub image: Option<Vec<u8>>,    
    pub msg_group_type: i32,
    // profile fields
    pub user_id: i64,
    pub user_name: String,
    pub full_name: String,
    pub avatar: Option<Vec<u8>>,
    // broadcast message fields
    pub broadcast_msg_id: Option<i64>,
    pub broadcast_msg_updated_at: Option<DateTime<Utc>>,
    pub broadcast_msg_body: Option<String>,
    pub broadcast_msg_likes: Option<i32>,
    pub broadcast_msg_image: Option<Vec<u8>>,    
    pub broadcast_msg_user_id: Option<i64>,
    pub broadcast_msg_user_name: Option<String>,
    pub broadcast_msg_full_name: Option<String>,
    pub broadcast_msg_avatar: Option<Vec<u8>>
}
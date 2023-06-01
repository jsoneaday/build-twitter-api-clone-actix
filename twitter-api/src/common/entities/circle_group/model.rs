use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Deserialize, Serialize, FromRow, Clone)]
pub struct CircleGroupQueryResult {
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub owner_id: i64
}

#[derive(Deserialize, Serialize, FromRow, Clone)]
pub struct CircleGroupWithProfileQueryResult {
    pub id: i64,
    pub updated_at: DateTime<Utc>,
    pub owner_id: i64,
    pub user_name: String,
    pub full_name: String,
    pub avatar: Vec<u8>
}

#[derive(Deserialize, Serialize, FromRow, Clone)]
pub struct CircleGroupMemberQueryResult {
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub circle_group_id: i64,
    pub member_id: i64
}

#[derive(Deserialize, Serialize, FromRow, Clone)]
pub struct CircleGroupMemberWithProfileQueryResult {
    pub id: i64,
    pub updated_at: DateTime<Utc>,
    pub circle_group_id: i64,
    pub member_id: i64,
    pub user_name: String,
    pub full_name: String,
    pub avatar: Vec<u8>
}
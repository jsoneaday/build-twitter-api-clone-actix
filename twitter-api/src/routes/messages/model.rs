use serde::{Deserialize, Serialize};
use serde_repr::*;
use chrono::prelude::*;
use crate::routes::profiles::model::ProfileShort;


#[derive(Deserialize)]
pub struct MessageQuery {
    pub id: i64
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageByFollowingQuery {
    pub follower_id: i64,
    pub last_updated_at: DateTime<Utc>,
    pub page_size: Option<i16>
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MessagePostJson {
    pub user_id: i64,
    pub body: String,
    pub group_type: GroupTypes,
    pub broadcasting_msg_id: Option<i64>
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageResponder {
    pub id: i64,
    pub updated_at: DateTime<Utc>,
    pub body: Option<String>,
    pub likes: i32,
    pub broadcasting_msg: Option<Box<MessageResponder>>,
    pub profile: ProfileShort
}

#[derive(Deserialize_repr, Serialize_repr, Clone)]
#[repr(i32)]
pub enum GroupTypes {
    Public = 1,
    Circle = 2
}

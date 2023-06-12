use actix_http::body::BoxBody;
use actix_web::{Responder, HttpResponse, HttpRequest, http::header::ContentType};
use serde::{Deserialize, Serialize};
use serde_repr::*;
use chrono::prelude::*;
use crate::routes::profiles::model::ProfileShort;
use std::vec::Vec;

#[derive(Deserialize)]
pub struct MessageQuery {
    pub id: i64
}

#[derive(Deserialize, Serialize)]
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
    pub group_type: MessageGroupTypes,
    pub broadcasting_msg_id: Option<i64>
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MessageResponder {
    pub id: i64,
    pub updated_at: DateTime<Utc>,
    pub body: Option<String>,
    pub likes: i32,
    pub broadcasting_msg: Option<Box<MessageResponder>>,
    pub profile: ProfileShort
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MessageResponders(pub Vec<MessageResponder>);

impl Responder for MessageResponder {
    type Body = BoxBody;

    fn respond_to(self, _: &HttpRequest) -> HttpResponse<Self::Body> {
        let body_result = serde_json::to_string(&self);

        match body_result {
            Ok(body) => {
                HttpResponse::Ok()
                .content_type(ContentType::json())
                .body(body)
            },
            Err(_) => {
                HttpResponse::InternalServerError()
                    .content_type(ContentType::json())
                    .body("Failed to serialize MessageResponder.")
            },
        }
    }
}

impl Responder for MessageResponders {
    type Body = BoxBody;

    fn respond_to(self, _: &HttpRequest) -> HttpResponse<Self::Body> {
        let body_result = serde_json::to_string(&self);

        match body_result {
            Ok(body) => {
                HttpResponse::Ok()
                .content_type(ContentType::json())
                .body(body)
            },
            Err(_) => {
                HttpResponse::InternalServerError()
                    .content_type(ContentType::json())
                    .body("Failed to serialize MessageResponders.")
            },
        }
    }
}

#[derive(Deserialize_repr, Serialize_repr, Clone)]
#[repr(i32)]
pub enum MessageGroupTypes {
    Public = 1,
    Circle = 2
}

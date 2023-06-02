use crate::common::entities::messages::model::{MessageWithFollowingAndBroadcastQueryResult};
use crate::common::app_state::AppState;
use crate::common::entities::messages::repo::{InsertMessageFn, QueryMessageFn, QueryMessagesFn};
use crate::routes::profiles::model::ProfileShort;
use actix_web::{web, web::{Path, Json}, Responder};
use std::error::Error;
use super::model::{MessageResponder, MessagePostJson, MessageQuery, MessageByFollowingQuery};


#[allow(unused)]
pub async fn create_message<T: InsertMessageFn>(app_data: web::Data<AppState<T>>, params: Json<MessagePostJson>) -> Result<impl Responder, Box<dyn Error>> {  
    let max = 141; 
    let body = if params.body.len() < max {
        &params.body[..]
    } else {
        &params.body[..max]
    };

    let group_type = params.group_type.clone() as i32;
    let result = app_data.db_repo.insert_message(params.user_id, body, group_type, params.broadcasting_msg_id).await;
    match result {
        Ok(id) => Ok(Json(id)),
        Err(e) => Err(Box::new(e))
    }
}

#[allow(unused)]
pub async fn get_message<T: QueryMessageFn>(app_data: web::Data<AppState<T>>, path: Path<MessageQuery>) -> Result<Option<impl Responder>, Box<dyn Error>> {
    let message_result = app_data.db_repo.query_message(path.id).await;

    match message_result {
        Ok(message) => {
            match message {
                Some(msg) => {
                    Ok(Some(Json(convert(&msg))))
                },
                None => Ok(None)
            }
        },
        Err(e) => Err(Box::new(e))
    }
}

#[allow(unused)]
pub async fn get_messages<T: QueryMessagesFn>(app_data: web::Data<AppState<T>>, path: Path<MessageByFollowingQuery>) -> Result<impl Responder, Box<dyn Error>>  {
    let page_size = match path.page_size {
        Some(ps) => ps,
        None => 10
    };
    let mut messages_result = app_data.db_repo.query_messages(
        path.follower_id, path.last_updated_at, page_size
    ).await;
    
    let mut msg_collection: Vec<MessageResponder> = vec![];
    match messages_result {
        Ok(messages) => {
            messages
                .iter()
                .for_each(|msg| {
                    msg_collection.push(convert(msg))
                });

            Ok(Json(msg_collection))
        },
        Err(e) => Err(Box::new(e))
    }
}

fn convert(message: &MessageWithFollowingAndBroadcastQueryResult) -> MessageResponder {
    MessageResponder {
        id: message.id,
        updated_at: message.updated_at,
        body: message.body.clone(),
        likes: message.likes,
        broadcasting_msg: match message.broadcast_msg_id {
            Some(id) => {
                Some(Box::new(MessageResponder { 
                    id,
                    updated_at: message.broadcast_msg_updated_at.unwrap(),
                    body: message.broadcast_msg_body.clone(),
                    likes: message.broadcast_msg_likes.unwrap(),
                    broadcasting_msg: None ,
                    profile: ProfileShort {
                        id: message.broadcast_msg_user_id.unwrap(),
                        user_name: message.broadcast_msg_user_name.clone().unwrap(),
                        full_name: message.broadcast_msg_full_name.clone().unwrap()
                    }
                }))
            },
            None => None
        },
        profile: ProfileShort {
            id: message.id,
            user_name: message.user_name.clone(),
            full_name: message.full_name.clone()
        }
    }
}

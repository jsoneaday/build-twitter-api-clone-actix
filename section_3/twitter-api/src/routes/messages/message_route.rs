use crate::common::entities::messages::model::{MessageWithFollowingAndBroadcastQueryResult};
use crate::common::app_state::AppState;
use crate::common::entities::messages::repo::{InsertMessageFn, QueryMessageFn, QueryMessagesFn};
use crate::routes::errors::error_utils::UserError;
use crate::routes::output_id::OutputId;
use crate::routes::profiles::model::ProfileShort;
use actix_web::{web, web::{Path, Json}};
use super::model::{MessageResponder, MessagePostJson, MessageQuery, MessageByFollowingQuery, MessageResponders};


#[allow(unused)]
pub async fn create_message<T: InsertMessageFn>(app_data: web::Data<AppState<T>>, params: Json<MessagePostJson>) -> Result<OutputId, UserError> {  
    let max = 141; 
    let body = if params.body.len() < max {
        &params.body[..]
    } else {
        &params.body[..max]
    };

    let group_type = params.group_type.clone() as i32;
    let result = app_data.db_repo.insert_message(params.user_id, body, group_type, params.broadcasting_msg_id).await;
    match result {
        Ok(id) => Ok(OutputId { id }),
        Err(e) => Err(e.into())
    }
}

#[allow(unused)]
pub async fn get_message<T: QueryMessageFn>(app_data: web::Data<AppState<T>>, path: Path<MessageQuery>) -> Result<Option<MessageResponder>, UserError> {
    let message_result = app_data.db_repo.query_message(path.id).await;

    match message_result {
        Ok(message) => {
            match message {
                Some(msg) => {
                    Ok(Some(convert(&msg)))
                },
                None => Ok(None)
            }
        },
        Err(e) => Err(e.into())
    }
}

#[allow(unused)]
pub async fn get_messages<T: QueryMessagesFn>(app_data: web::Data<AppState<T>>, path: Json<MessageByFollowingQuery>) -> Result<MessageResponders, UserError>  {
    println!("start get_messages");
    let page_size = match path.page_size {
        Some(ps) => ps,
        None => 10
    };
    
    let mut messages_result = app_data.db_repo.query_messages(
        path.follower_id, path.last_updated_at, page_size
    ).await;
    println!("end get_messages");
    
    let mut msg_collection: Vec<MessageResponder> = vec![];
    match messages_result {
        Ok(messages) => {
            messages
                .iter()
                .for_each(|msg| {
                    msg_collection.push(convert(msg))
                });

            Ok(MessageResponders(msg_collection))
        },
        Err(e) => Err(e.into())
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


#[cfg(test)]
mod tests {
    use actix_web::web::Json;
    use async_trait::async_trait;
    use crate::{common::entities::messages::repo::InsertMessageFn, routes::messages::{message_route::create_message, model::MessagePostJson}, common_tests::actix_fixture::{get_app_data, get_fake_message_body}};
    

    mod test_mod_create_message_and_check_id {        
        use super::*;

        const ID: i64 = 22;
        struct TestRepo;
        
        #[allow(unused)]
        #[async_trait]
        impl InsertMessageFn for TestRepo {            
            async fn insert_message(
                &self,
                user_id: i64,
                body: &str,
                group_type: i32,
                broadcasting_msg_id: Option<i64>
            ) -> Result<i64, sqlx::Error> {
                Ok(ID)
            }
        }

        #[tokio::test]
        async fn test_create_message_and_check_id() {
            let repo = TestRepo;
            let app_data = get_app_data(repo).await;

            let result = create_message(app_data, Json(
                MessagePostJson{ user_id: 0, body: get_fake_message_body(None), group_type: crate::routes::messages::model::MessageGroupTypes::Circle, broadcasting_msg_id: None }
            )).await;

            assert!(!result.is_err());
            assert!(result.ok().unwrap().id == ID);
        }
    }

    mod test_mod_create_message_failure_returns_correct_error {      
        use crate::routes::errors::error_utils::UserError;
        use super::*;

        struct TestRepo;
        
        #[allow(unused)]
        #[async_trait]
        impl InsertMessageFn for TestRepo {            
            async fn insert_message(
                &self,
                user_id: i64,
                body: &str,
                group_type: i32,
                broadcasting_msg_id: Option<i64>
            ) -> Result<i64, sqlx::Error> {
                Err(sqlx::Error::PoolTimedOut)
            }
        }

        #[tokio::test]
        async fn test_create_message_failure_returns_correct_error () {
            let repo = TestRepo;
            let app_data = get_app_data(repo).await;

            let result = create_message(app_data, Json(
                MessagePostJson{ user_id: 0, body: get_fake_message_body(None), group_type: crate::routes::messages::model::MessageGroupTypes::Circle, broadcasting_msg_id: None }
            )).await;

            assert!(result.is_err());
            assert!(result.err().unwrap() == UserError::InternalError);
        }
    }

    mod test_mod_get_message_failure_returns_correct_error {      
        use actix_web::web::Path;
        use crate::{routes::{errors::error_utils::UserError, messages::{message_route::get_message, model::MessageQuery}}, common::entities::messages::{repo::QueryMessageFn, model::MessageWithFollowingAndBroadcastQueryResult}};
        use super::*;

        struct TestRepo;
        
        #[allow(unused)]
        #[async_trait]
        impl QueryMessageFn for TestRepo {            
            async fn query_message(&self, id: i64) -> Result<Option<MessageWithFollowingAndBroadcastQueryResult>, sqlx::Error> {
                Err(sqlx::Error::ColumnNotFound("na".to_string()))
            }
        }

        #[tokio::test]
        async fn test_get_message_failure_returns_correct_error () {
            let repo = TestRepo;
            let app_data = get_app_data(repo).await;

            let result = get_message(app_data, Path::from(MessageQuery{ id: 0 })).await;

            assert!(result.is_err());
            assert!(result.err().unwrap() == UserError::InternalError);
        }
    }

    mod test_mod_get_message_and_check_id {      
        use actix_web::web::Path;
        use chrono::Utc;
        use fake::faker::{internet::en::Username, name::en::{FirstName, LastName}};
        use fake::Fake;
        use crate::{
            routes::{messages::{message_route::get_message, model::{MessageQuery, MessageGroupTypes}}}, 
            common::entities::messages::{repo::QueryMessageFn, model::MessageWithFollowingAndBroadcastQueryResult}
        };
        use super::*;

        const ID: i64 = 22;
        struct TestRepo;
        
        #[allow(unused)]
        #[async_trait]
        impl QueryMessageFn for TestRepo {            
            async fn query_message(&self, id: i64) -> Result<Option<MessageWithFollowingAndBroadcastQueryResult>, sqlx::Error> {
                Ok(Some(
                    MessageWithFollowingAndBroadcastQueryResult {
                        id: ID,
                        updated_at: Utc::now(),
                        body: None,
                        likes: 1,
                        image: None,
                        msg_group_type: MessageGroupTypes::Public as i32,
                        user_id: 0,
                        user_name: Username().fake(),
                        full_name: format!("{} {}", FirstName().fake::<String>(), LastName().fake::<String>()),
                        avatar: None,
                        broadcast_msg_id: None,
                        broadcast_msg_updated_at: None,
                        broadcast_msg_body: None,
                        broadcast_msg_likes: None,
                        broadcast_msg_image: None,
                        broadcast_msg_user_id: None,
                        broadcast_msg_user_name: None,
                        broadcast_msg_full_name: None,
                        broadcast_msg_avatar: None,
                    }
                ))
            }
        }

        #[tokio::test]
        async fn test_get_message_and_check_id() {
            let repo = TestRepo;
            let app_data = get_app_data(repo).await;

            let result = get_message(app_data, Path::from(MessageQuery{ id: 0 })).await;

            assert!(!result.is_err());
            assert!(result.ok().unwrap().unwrap().id == ID);
        }
    }

    mod test_mod_get_messages_failure_returns_correct_error {    
        use chrono::{DateTime, Utc};
        use crate::{
            routes::{errors::error_utils::UserError, messages::{message_route::get_messages, model::MessageByFollowingQuery}}, 
            common::entities::messages::{repo::QueryMessagesFn, model::MessageWithFollowingAndBroadcastQueryResult}
        };
        use super::*;

        struct TestRepo;
        
        #[allow(unused)]
        #[async_trait]
        impl QueryMessagesFn for TestRepo {            
            async fn query_messages(
                &self, 
                user_id: i64,
                last_updated_at: DateTime<Utc>,
                page_size: i16
            ) -> Result<Vec<MessageWithFollowingAndBroadcastQueryResult>, sqlx::Error> {
                Err(sqlx::Error::ColumnNotFound("na".to_string()))
            }
        }

        #[tokio::test]
        async fn test_get_messages_failure_returns_correct_error () {
            let repo = TestRepo;
            let app_data = get_app_data(repo).await;

            let result = get_messages(app_data, Json(MessageByFollowingQuery { follower_id: 0, last_updated_at: Utc::now(), page_size: None })).await;

            assert!(result.is_err());
            assert!(result.err().unwrap() == UserError::InternalError);
        }
    }

    mod test_mod_get_messages_and_check_id {  
        use chrono::{Utc, DateTime};
        use fake::faker::{internet::en::Username, name::en::{FirstName, LastName}};
        use fake::Fake;
        use crate::{
            routes::{messages::{message_route::get_messages, model::{MessageGroupTypes, MessageByFollowingQuery}}}, 
            common::entities::messages::{repo::QueryMessagesFn, model::MessageWithFollowingAndBroadcastQueryResult}
        };
        use super::*;

        const ID: i64 = 22;
        struct TestRepo;
        
        #[allow(unused)]
        #[async_trait]
        impl QueryMessagesFn for TestRepo {            
            async fn query_messages(
                &self, 
                user_id: i64,
                last_updated_at: DateTime<Utc>,
                page_size: i16) -> Result<Vec<MessageWithFollowingAndBroadcastQueryResult>, sqlx::Error> {
                Ok(vec![
                    MessageWithFollowingAndBroadcastQueryResult {
                        id: ID,
                        updated_at: Utc::now(),
                        body: None,
                        likes: 1,
                        image: None,
                        msg_group_type: MessageGroupTypes::Public as i32,
                        user_id: 0,
                        user_name: Username().fake(),
                        full_name: format!("{} {}", FirstName().fake::<String>(), LastName().fake::<String>()),
                        avatar: None,
                        broadcast_msg_id: None,
                        broadcast_msg_updated_at: None,
                        broadcast_msg_body: None,
                        broadcast_msg_likes: None,
                        broadcast_msg_image: None,
                        broadcast_msg_user_id: None,
                        broadcast_msg_user_name: None,
                        broadcast_msg_full_name: None,
                        broadcast_msg_avatar: None,
                    }
                ])
            }
        }

        #[tokio::test]
        async fn test_get_messages_and_check_id() {
            let repo = TestRepo;
            let app_data = get_app_data(repo).await;

            let result = get_messages(app_data, Json(MessageByFollowingQuery { follower_id: 0, last_updated_at: Utc::now(), page_size: None })).await;

            assert!(!result.is_err());
            assert!(result.ok().unwrap().0[0].id == ID);
        }
    }
}
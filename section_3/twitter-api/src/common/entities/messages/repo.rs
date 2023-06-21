use crate::common::entities::{ base::{ EntityId, DbRepo, DbConnGetter } };
use mockall::automock;
use sqlx::{ Pool, Postgres };
use super::model::MessageWithFollowingAndBroadcastQueryResult;
use async_trait::async_trait;
use chrono::{ DateTime, Utc };

// 1. we create a single logical container where multiple related members can exist
// 2. we create repeatable structure to our code
// 3. we can hide some members even from our parent module
mod private_members {
    use crate::common::entities::messages::model::MessageWithProfileQueryResult;
    use super::*;

    pub async fn insert_message_inner(
        conn: &Pool<Postgres>,
        user_id: i64,
        body: &str,
        group_type: i32,
        broadcasting_msg_id: Option<i64>
    ) -> Result<i64, sqlx::Error> {
        let mut tx = conn.begin().await.unwrap();

        let insert_msg_result = sqlx
            ::query_as::<_, EntityId>(
                "insert into message (user_id, body, msg_group_type) values ($1, $2, $3) returning id"
            )
            .bind(user_id)
            .bind(body)
            .bind(group_type)
            .fetch_one(&mut tx).await;

        let message_id_result = match insert_msg_result {
            Ok(r) => Ok(r.id),
            Err(e) => {
                println!("insert_message error: {}", e);
                Err(e)
            }
        };
        if message_id_result.is_err() {
            return message_id_result;
        }

        if let Some(bm_id) = broadcasting_msg_id {
            let message_broadcast_result = sqlx
                ::query_as::<_, EntityId>(
                    "insert into message_broadcast (main_msg_id, broadcasting_msg_id) values ($1, $2) returning id"
                )
                .bind(message_id_result.as_ref().unwrap())
                .bind(bm_id)
                .fetch_one(&mut tx).await;

            if message_broadcast_result.is_err() {
                _ = tx.rollback().await;
                return Err(message_broadcast_result.err().unwrap());
            }
        }

        _ = tx.commit().await;

        message_id_result
    }

    pub async fn insert_response_message_inner(
        conn: &Pool<Postgres>,
        user_id: i64,
        body: &str,
        group_type: i32,
        original_msg_id: i64
    ) -> Result<i64, sqlx::Error> {
        let mut tx = conn.begin().await.unwrap();

        let insert_result = sqlx
            ::query_as::<_, EntityId>(
                "insert into message (user_id, body, msg_group_type) values ($1, $2, $3) returning id"
            )
            .bind(user_id)
            .bind(body)
            .bind(group_type)
            .fetch_one(&mut tx).await;
        let msg_id_result = match insert_result {
            Ok(r) => Ok(r.id),
            Err(e) => {
                println!("insert_message error: {}", e);
                Err(e)
            }
        };
        if msg_id_result.is_err() {
            return msg_id_result;
        }
        let msg_id: i64 = msg_id_result.unwrap();

        let insert_msg_response_result = sqlx
            ::query_as::<_, EntityId>(
                "insert into message_response (original_msg_id, responding_msg_id) values ($1, $2) returning id"
            )
            .bind(original_msg_id)
            .bind(msg_id)
            .fetch_one(&mut tx).await;

        let msg_response_id_result = match insert_msg_response_result {
            Ok(row) => Ok(row.id),
            Err(e) => Err(e),
        };
        if msg_response_id_result.is_err() {
            _ = tx.rollback().await;
            return msg_response_id_result;
        }

        _ = tx.commit().await;

        Ok(msg_id)
    }

    pub async fn query_message_inner(
        conn: &Pool<Postgres>,
        id: i64
    ) -> Result<Option<MessageWithFollowingAndBroadcastQueryResult>, sqlx::Error> {
        let message_result = sqlx
            ::query_as::<_, MessageWithProfileQueryResult>(
                r"
                select m.id, m.updated_at, m.body, m.likes, m.image, m.msg_group_type, m.user_id, p.user_name, p.full_name, p.avatar, mb.id as broadcast_msg_id                    
                    from message m 
                        join profile p on m.user_id = p.id
                        left join message_broadcast mb on m.id = mb.main_msg_id
                    where
                        m.id = $1
            "
            )
            .bind(id)
            .fetch_optional(conn).await;

        match message_result {
            Ok(message) => {
                if let Some(msg) = message {
                    let optional_matching_broadcast_message = get_broadcasting_message_of_message(
                        conn,
                        &msg
                    ).await;
                    let final_message = append_broadcast_msg_to_msg(
                        optional_matching_broadcast_message.as_ref(),
                        &msg
                    );
                    Ok(Some(final_message))
                } else {
                    Ok(None)
                }
            }
            Err(e) => Err(e),
        }
    }

    pub async fn query_messages_inner(
        conn: &Pool<Postgres>,
        user_id: i64,
        last_updated_at: DateTime<Utc>,
        page_size: i16
    ) -> Result<Vec<MessageWithFollowingAndBroadcastQueryResult>, sqlx::Error> {
        let following_messages_with_profiles_result = sqlx
            ::query_as::<_, MessageWithProfileQueryResult>(
                r"
                select m.id, m.updated_at, m.body, m.likes, m.image, m.msg_group_type, m.user_id, p.user_name, p.full_name, p.avatar, mb.id as broadcast_msg_id                    
                    from message m 
                        join follow f on m.user_id = f.following_id
                        join profile p on p.id = f.following_id
                        left join message_broadcast mb on m.id = mb.main_msg_id
                        where
                            f.follower_id = $1 
                            and m.updated_at < $2
                        order by m.updated_at desc 
                        limit $3
            "
            )
            .bind(user_id)
            .bind(last_updated_at)
            .bind(page_size)
            .fetch_all(conn).await;

        match following_messages_with_profiles_result {
            Ok(following_messages) => {
                let following_messages_with_broadcasts = following_messages
                    .clone()
                    .into_iter()
                    .filter(|msg| {
                        msg.broadcast_msg_id.is_some() && msg.broadcast_msg_id.unwrap() > 0
                    })
                    .collect::<Vec<MessageWithProfileQueryResult>>();

                let optional_matching_broadcast_messages = get_broadcasting_messages_of_messages(
                    conn,
                    &following_messages_with_broadcasts
                ).await;
                let final_message_list = append_broadcast_msgs_to_msgs(
                    &optional_matching_broadcast_messages,
                    following_messages
                );
                Ok(final_message_list)
            }
            Err(e) => Err(e),
        }
    }

    async fn get_broadcasting_messages_of_messages(
        conn: &Pool<Postgres>,
        following_messages_with_broadcasts: &Vec<MessageWithProfileQueryResult>
    ) -> Option<Vec<MessageWithProfileQueryResult>> {
        let following_broadcast_message_ids = following_messages_with_broadcasts
            .iter()
            .map(|msg| { msg.broadcast_msg_id.unwrap() })
            .collect::<Vec<i64>>();

        let broadcasting_msg_result = sqlx
            ::query_as::<_, MessageWithProfileQueryResult>(
                r"
                select m.id, m.updated_at, m.body, m.likes, m.image, m.msg_group_type, m.user_id, p.user_name, p.full_name, p.avatar, mb.id as broadcast_msg_id
                    from message m 
                        join profile p on m.user_id = p.id
                        left join message_broadcast mb on m.id = mb.main_msg_id
                    where m.id = ANY($1)
            "
            )
            .bind(following_broadcast_message_ids)
            .fetch_all(conn).await;

        match broadcasting_msg_result {
            Ok(broadcast_messages) => { Some(broadcast_messages) }
            Err(e) => {
                println!("get_broadcasting_messages_of_messages: {}", e);
                None
            }
        }
    }

    async fn get_broadcasting_message_of_message(
        conn: &Pool<Postgres>,
        message: &MessageWithProfileQueryResult
    ) -> Option<MessageWithProfileQueryResult> {
        let broadcasting_msg_result = sqlx
            ::query_as::<_, MessageWithProfileQueryResult>(
                r"
                select m.id, m.updated_at, m.body, m.likes, m.image, m.msg_group_type, m.user_id, p.user_name, p.full_name, p.avatar, mb.id as broadcast_msg_id
                    from message m 
                        join profile p on m.user_id = p.id
                        left join message_broadcast mb on m.id = mb.main_msg_id
                    where m.id = $1
            "
            )
            .bind(message.broadcast_msg_id)
            .fetch_optional(conn).await;

        match broadcasting_msg_result {
            Ok(broadcast_message) => { broadcast_message }
            Err(e) => {
                println!("get_broadcasting_messages_of_messages: {}", e);
                None
            }
        }
    }

    fn append_broadcast_msgs_to_msgs(
        optional_broadcast_messages: &Option<Vec<MessageWithProfileQueryResult>>,
        following_messages_with_broadcasts: Vec<MessageWithProfileQueryResult>
    ) -> Vec<MessageWithFollowingAndBroadcastQueryResult> {
        let mut final_list_of_messages: Vec<MessageWithFollowingAndBroadcastQueryResult> = vec![];

        following_messages_with_broadcasts.iter().for_each(|following_message_with_broadcast| {
            let matching_broadcast_msg = if
                let Some(broadcast_messages) = optional_broadcast_messages
            {
                broadcast_messages
                    .iter()
                    .find(|bm| { Some(bm.id) == following_message_with_broadcast.broadcast_msg_id })
            } else {
                None
            };

            final_list_of_messages.push(
                append_broadcast_msg_to_msg(
                    matching_broadcast_msg,
                    following_message_with_broadcast
                )
            );
        });

        final_list_of_messages
    }

    fn append_broadcast_msg_to_msg(
        broadcast_message: Option<&MessageWithProfileQueryResult>,
        message_with_broadcast: &MessageWithProfileQueryResult
    ) -> MessageWithFollowingAndBroadcastQueryResult {
        let mut final_message = MessageWithFollowingAndBroadcastQueryResult {
            id: message_with_broadcast.id,
            updated_at: message_with_broadcast.updated_at,
            body: message_with_broadcast.body.clone(),
            likes: message_with_broadcast.likes,
            image: message_with_broadcast.image.clone(),
            msg_group_type: message_with_broadcast.msg_group_type,
            user_id: message_with_broadcast.user_id,
            user_name: message_with_broadcast.user_name.clone(),
            full_name: message_with_broadcast.full_name.clone(),
            avatar: message_with_broadcast.avatar.clone(),
            broadcast_msg_id: None,
            broadcast_msg_updated_at: None,
            broadcast_msg_user_id: None,
            broadcast_msg_body: None,
            broadcast_msg_likes: None,
            broadcast_msg_image: None,
            broadcast_msg_user_name: None,
            broadcast_msg_full_name: None,
            broadcast_msg_avatar: None,
        };

        if let Some(matching_broadcast) = broadcast_message {
            final_message.broadcast_msg_id = Some(matching_broadcast.id);
            final_message.broadcast_msg_updated_at = Some(matching_broadcast.updated_at);
            final_message.broadcast_msg_body = matching_broadcast.body.to_owned();
            final_message.broadcast_msg_likes = Some(matching_broadcast.likes);
            final_message.broadcast_msg_image = matching_broadcast.image.to_owned();
            final_message.broadcast_msg_user_id = Some(matching_broadcast.user_id);
            final_message.broadcast_msg_user_name = Some(matching_broadcast.user_name.to_string());
            final_message.broadcast_msg_full_name = Some(matching_broadcast.full_name.to_string());
            final_message.broadcast_msg_avatar = matching_broadcast.avatar.to_owned();
        }

        final_message
    }
}

#[automock]
#[async_trait]
pub trait InsertMessageFn {
    async fn insert_message(
        &self,
        user_id: i64,
        body: &str,
        group_type: i32,
        broadcasting_msg_id: Option<i64>
    ) -> Result<i64, sqlx::Error>;
}

#[async_trait]
impl InsertMessageFn for DbRepo {
    async fn insert_message(
        &self,
        user_id: i64,
        body: &str,
        group_type: i32,
        broadcasting_msg_id: Option<i64>
    ) -> Result<i64, sqlx::Error> {
        private_members::insert_message_inner(
            self.get_conn(),
            user_id,
            body,
            group_type,
            broadcasting_msg_id
        ).await
    }
}

#[automock]
#[async_trait]
pub trait InsertResponseMessageFn {
    async fn insert_response_message(
        &self,
        user_id: i64,
        body: &str,
        group_type: i32,
        original_msg_id: i64
    ) -> Result<i64, sqlx::Error>;
}

#[async_trait]
impl InsertResponseMessageFn for DbRepo {
    async fn insert_response_message(
        &self,
        user_id: i64,
        body: &str,
        group_type: i32,
        original_msg_id: i64
    ) -> Result<i64, sqlx::Error> {
        private_members::insert_response_message_inner(
            self.get_conn(),
            user_id,
            body,
            group_type,
            original_msg_id
        ).await
    }
}

#[automock]
#[async_trait]
pub trait QueryMessageFn {
    async fn query_message(
        &self,
        id: i64
    ) -> Result<Option<MessageWithFollowingAndBroadcastQueryResult>, sqlx::Error>;
}

#[async_trait]
impl QueryMessageFn for DbRepo {
    async fn query_message(
        &self,
        id: i64
    ) -> Result<Option<MessageWithFollowingAndBroadcastQueryResult>, sqlx::Error> {
        private_members::query_message_inner(self.get_conn(), id).await
    }
}

#[automock]
#[async_trait]
pub trait QueryMessagesFn {
    async fn query_messages(
        &self,
        user_id: i64,
        last_updated_at: DateTime<Utc>,
        page_size: i16
    ) -> Result<Vec<MessageWithFollowingAndBroadcastQueryResult>, sqlx::Error>;
}

#[async_trait]
impl QueryMessagesFn for DbRepo {
    async fn query_messages(
        &self,
        user_id: i64,
        last_updated_at: DateTime<Utc>,
        page_size: i16
    ) -> Result<Vec<MessageWithFollowingAndBroadcastQueryResult>, sqlx::Error> {
        private_members::query_messages_inner(self.get_conn(), user_id, last_updated_at, page_size).await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{ Arc, RwLock };
    use fake::{ faker::name::en::{ Name, FirstName, LastName }, Fake };
    use lazy_static::lazy_static;
    use crate::{
        common_tests::actix_fixture::PUBLIC_GROUP_TYPE,
        common::entities::{profiles::{
                repo::{ InsertProfileFn, QueryProfileFn, MockInsertProfileFn },
                model::ProfileCreate,
            }
        }
    };
    use super::*;

    #[derive(Clone)]
    struct Fixtures {
        pub original_msg_id: i64,
        pub profile_id: i64,
        pub profile_create: ProfileCreate,
        pub db_repo: DbRepo
    }

    const PREFIX: &str = "Test message";

    lazy_static! {
        static ref FIXTURES: Arc<RwLock<Option<Fixtures>>> = Arc::new(RwLock::new(None));
    }

    async fn setup_data(db_repo: DbRepo) -> Fixtures {
        let first_name: String = FirstName().fake();
        let last_name: String = LastName().fake();
        let profile_create = ProfileCreate {
            user_name: Name().fake(),
            full_name: format!("{} {}", first_name, last_name),
            description: format!("{} a description", PREFIX),
            region: Some("usa".to_string()),
            main_url: Some("http://whatever.com".to_string()),
            avatar: Some(vec![]),
        };
        let profile = db_repo.insert_profile(profile_create.clone()).await;
        let profile_id = profile.unwrap();
        let original_msg_id = db_repo
            .insert_message(profile_id, "Testing body 123", PUBLIC_GROUP_TYPE, None).await
            .unwrap();

        Fixtures {
            original_msg_id,
            profile_id,
            profile_create,
            db_repo,
        }
    }

    async fn setup_fixtures() {
        let fixtures = Arc::clone(&FIXTURES);
        let mut fx = fixtures.write().unwrap();
        match fx.clone() {
            Some(_) => (),
            None => {
                let db_repo = DbRepo::init().await;

                *fx = Some(setup_data(db_repo).await);
            }
        }
    }

    lazy_static! {
        static ref RT: tokio::runtime::Runtime = {
            let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();

            rt.block_on(async {
                setup_fixtures().await;
            });

            rt
        };
    }

    fn get_fixtures() -> Fixtures {
        Arc::clone(&FIXTURES).read().unwrap().clone().unwrap()
    }

    fn get_insert_profile_mock() -> MockInsertProfileFn {
        let mut mock_insert_profile = MockInsertProfileFn::new();
        mock_insert_profile
            .expect_insert_profile()
            .returning(move |_| { Ok(get_fixtures().profile_id) });
        mock_insert_profile
    }

    fn get_insert_message_mock() -> MockInsertMessageFn {
        let mut mock_insert_message = MockInsertMessageFn::new();
        mock_insert_message
            .expect_insert_message()
            .returning(|_, _, _, _| { Ok(get_fixtures().original_msg_id) });
        mock_insert_message
    }

    mod test_mod_insert_message {
        use super::*;

        async fn test_insert_message_body() {
            let fixtures = get_fixtures();

            let mock_insert_profile = get_insert_profile_mock();

            let profile_id = mock_insert_profile
                .insert_profile(ProfileCreate {
                    user_name: "tester".to_string(),
                    full_name: "Dave Wave".to_string(),
                    description: format!("{} a description", PREFIX),
                    region: Some("usa".to_string()),
                    main_url: Some("http://whatever.com".to_string()),
                    avatar: Some(vec![]),
                }).await
                .unwrap();

            let original_msg_id = fixtures.db_repo
                .insert_message(
                    profile_id,
                    "Body of message that is being responded to.",
                    PUBLIC_GROUP_TYPE,
                    None
                ).await
                .unwrap();

            assert!(original_msg_id > 0);
        }

        #[test]
        fn test_insert_message() {
            RT.block_on(test_insert_message_body())
        }
    }

    mod test_mod_query_message {
        use super::*;

        async fn test_query_message_body() {
            let fixtures = get_fixtures();
            let mock_insert_profile = get_insert_profile_mock();
            let mock_insert_message = get_insert_message_mock();

            let profile_id = mock_insert_profile
                .insert_profile(fixtures.profile_create.clone()).await
                .unwrap();

            let original_msg_id = mock_insert_message
                .insert_message(
                    profile_id,
                    "Body of message that is being responded to.",
                    PUBLIC_GROUP_TYPE,
                    None
                ).await
                .unwrap();

            let original_message = fixtures.db_repo
                .query_message(original_msg_id).await
                .unwrap()
                .unwrap();

            assert!(original_message.id == original_msg_id);
        }

        #[test]
        fn test_query_message() {
            RT.block_on(test_query_message_body())
        }
    }

    mod test_mod_insert_response_message {
        use super::*;

        async fn test_insert_response_message_body() {
            let fixtures = get_fixtures();
            let mock_insert_profile = get_insert_profile_mock();
            let mock_insert_message = get_insert_message_mock();

            let profile_id_result = mock_insert_profile.insert_profile(
                ProfileCreate {
                    user_name: "tester".to_string(),
                    full_name: "Dave Wave".to_string(),
                    description: "a description".to_string(),
                    region: Some("usa".to_string()),
                    main_url: Some("http://whatever.com".to_string()),
                    avatar: Some(vec![]),
                }
            ).await;
            let profile_id = profile_id_result.unwrap();
            let original_msg_id = mock_insert_message.insert_message(
                profile_id,
                "Body of message that is being responded to.",
                PUBLIC_GROUP_TYPE,
                None
            ).await;

            let response_msg = fixtures.db_repo.insert_response_message(
                profile_id,
                "Body of response message",
                PUBLIC_GROUP_TYPE,
                original_msg_id.unwrap()
            ).await;
            assert!(response_msg.unwrap() > 0);
        }

        #[test]
        fn test_insert_response_message() {
            RT.block_on(test_insert_response_message_body())
        }
    }

    // this section shows that by using modules we are able to separate concerns and provide each test with
    // whatever data it may need uniquely
    mod test_mod_query_messages_by_following {
        use crate::common::entities::profiles::{ model::ProfileQueryResult, repo::{FollowUserFn, MockFollowUserFn} };
        use super::*;

        #[derive(Clone)]
        struct QueryMsgFollowingFixtures {
            pub follower_user: ProfileQueryResult,
            pub following_users: Vec<ProfileQueryResult>,
            pub following_users_messages: Vec<MessageWithFollowingAndBroadcastQueryResult>,
            pub db_repo: DbRepo,
        }

        lazy_static! {
            static ref LOCAL_FIXTURES: Arc<RwLock<Option<QueryMsgFollowingFixtures>>> = Arc::new(RwLock::new(None));
        }

        async fn setup(db_repo: DbRepo) -> QueryMsgFollowingFixtures {
            let follower_id = db_repo
                .insert_profile(ProfileCreate {
                    user_name: "follower".to_string(),
                    full_name: "Dave Follower".to_string(),
                    description: "Follower description".to_string(),
                    region: Some("usa".to_string()),
                    main_url: Some("http://whatever.com".to_string()),
                    avatar: Some(vec![]),
                }).await
                .unwrap();

            let follower_user = db_repo.query_profile(follower_id).await.unwrap().unwrap();

            let mut following_users: Vec<ProfileQueryResult> = Vec::new();
            let mut following_users_messages: Vec<MessageWithFollowingAndBroadcastQueryResult> =
                Vec::new();
            let following_letters = vec!["a", "b"];
            for l in following_letters {
                let following_id = db_repo
                    .insert_profile(ProfileCreate {
                        user_name: format!("following_{}", l),
                        full_name: format!("Dave Following{}", l),
                        description: format!("Follower{} description", l),
                        region: Some("usa".to_string()),
                        main_url: Some("http://whatever.com".to_string()),
                        avatar: Some(vec![]),
                    }).await
                    .unwrap();

                following_users.push(
                    db_repo.query_profile(following_id).await.unwrap().unwrap()
                );

                let following_user_message_1_id = db_repo
                    .insert_message(
                        following_id,
                        format!("Message {}: 1", l).as_str(),
                        PUBLIC_GROUP_TYPE,
                        None
                    ).await
                    .unwrap();
                let following_user_message_1 = db_repo
                    .query_message(following_user_message_1_id).await
                    .unwrap()
                    .unwrap();
                following_users_messages.push(following_user_message_1);

                let following_user_message_2_id = db_repo
                    .insert_message(
                        following_id,
                        format!("Message {}: 2", l).as_str(),
                        PUBLIC_GROUP_TYPE,
                        None
                    ).await
                    .unwrap();
                let following_user_message_2 = db_repo
                    .query_message(following_user_message_2_id).await
                    .unwrap()
                    .unwrap();
                following_users_messages.push(following_user_message_2);

                _ = db_repo.follow_user(follower_id, following_id).await.unwrap();
            }

            QueryMsgFollowingFixtures {
                follower_user,
                following_users,
                following_users_messages,
                db_repo,
            }
        }

        async fn setup_fixtures() {
            let fixtures = Arc::clone(&LOCAL_FIXTURES);
            let mut fx = fixtures.write().unwrap();
            match fx.clone() {
                Some(_) => (),
                None => {
                    let db_repo = DbRepo::init().await;
                    *fx = Some(setup(db_repo.clone()).await);
                }
            }
        }

        async fn get_local_fixtures() -> QueryMsgFollowingFixtures {
            Arc::clone(&LOCAL_FIXTURES).read().unwrap().clone().unwrap()
        }

        #[tokio::test]
        async fn test_query_messages_by_following() {
            setup_fixtures().await;
            let insert_profile_fixtures = get_local_fixtures().await;
            let insert_message_fixtures = get_local_fixtures().await;
            let query_messages_fixtures = get_local_fixtures().await;

            // create a single profile that will follow other profiles
            let mut mock_insert_profile = MockInsertProfileFn::new();
            mock_insert_profile
                .expect_insert_profile()
                .returning(move |params| {
                    if params.user_name.eq(&insert_profile_fixtures.follower_user.user_name) {
                        Ok(insert_profile_fixtures.follower_user.id)
                    } else {
                        let following = &insert_profile_fixtures.following_users
                            .iter()
                            .find(|fl| { fl.user_name == params.user_name });
                        Ok(following.unwrap().id)
                    }
                 });
            let mut mock_insert_message = MockInsertMessageFn::new();
            mock_insert_message
                .expect_insert_message()
                .returning(move |user_id, body, _, _| {
                    Ok(
                        insert_message_fixtures
                            .following_users_messages.clone()
                            .into_iter()
                            .find(|msg| {
                                msg.user_id == user_id && msg.body == Some(body.to_string())
                            })
                            .unwrap().id
                    )
                });
            let mut mock_follow_user = MockFollowUserFn::new();
            mock_follow_user
                .expect_follow_user()
                .returning(|_, _| { Ok(0) });
                 
            let follower_id = mock_insert_profile
                .insert_profile(ProfileCreate {
                    user_name: "follower".to_string(),
                    full_name: "Dave Follower".to_string(),
                    description: "Follower description".to_string(),
                    region: Some("usa".to_string()),
                    main_url: Some("http://whatever.com".to_string()),
                    avatar: Some(vec![]),
                }).await
                .unwrap();

            let mut created_following_messages: Vec<i64> = vec![];
            for l in ["a", "b"] {
                let following_id = mock_insert_profile
                    .insert_profile(ProfileCreate {
                        user_name: format!("following_{}", l),
                        full_name: format!("Dave Following{}", l),
                        description: format!("Follower{} description", l),
                        region: Some("usa".to_string()),
                        main_url: Some("http://whatever.com".to_string()),
                        avatar: Some(vec![]),
                    }).await
                    .unwrap();

                // create several messages by those following profiles
                let following_message_1_id = mock_insert_message
                    .insert_message(
                        following_id,
                        format!("Message {}: 1", l).as_str(),
                        PUBLIC_GROUP_TYPE,
                        None
                    ).await
                    .unwrap();
                created_following_messages.push(following_message_1_id);
                let following_message_2_id = mock_insert_message
                    .insert_message(
                        following_id,
                        format!("Message {}: 2", l).as_str(),
                        PUBLIC_GROUP_TYPE,
                        None
                    ).await
                    .unwrap();
                created_following_messages.push(following_message_2_id);

                // set follow
                _ = mock_follow_user.follow_user(follower_id, following_id).await;
            }

            // query db to get the messages created by profiles the single user is following
            let following_messages = query_messages_fixtures.db_repo
                .query_messages(follower_id, Utc::now(), 10).await
                .unwrap();
            let following_msg_ids = following_messages
                .iter()
                .map(|fm| { fm.id })
                .collect::<Vec<i64>>();

            for following_msg_id in &following_msg_ids {
                assert!(created_following_messages.contains(following_msg_id));
            }
            assert!(following_msg_ids.len() == created_following_messages.len());
        }
    }
}

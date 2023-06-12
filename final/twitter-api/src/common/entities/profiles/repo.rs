use crate::common::entities::base::{ EntityId, DbRepo, DbConnGetter };
use super::model::{ ProfileCreate, ProfileQueryResult };
use async_trait::async_trait;
use sqlx::{ Pool, Postgres };
use mockall::automock;
use mockall::predicate::*;

mod private_members {
    use super::*;

    pub async fn insert_profile_inner(
        conn: &Pool<Postgres>,
        params: ProfileCreate
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx
            ::query_as::<_, EntityId>(
                r"
                insert into Profile 
                    (user_name, full_name, description, region, main_url, avatar) 
                    values 
                    ($1, $2, $3, $4, $5, $6)
                returning id"
            )
            .bind(&params.user_name)
            .bind(&params.full_name)
            .bind(&params.description)
            .bind(&params.region)
            .bind(&params.main_url)
            .bind(&params.avatar)
            .fetch_one(conn).await;

        match result {
            Ok(r) => Ok(r.id),
            Err(e) => {
                println!("create_profile error: {}", e);
                Err(e)
            }
        }
    }

    pub async fn update_profile_avatar_inner(
        conn: &Pool<Postgres>,
        user_id: i64,
        avatar: Vec<u8>
    ) -> Result<(), sqlx::Error> {
        let update_result = sqlx
            ::query::<_>("update profile set avatar = $1 where id = $2")
            .bind(avatar)
            .bind(user_id)
            .execute(conn).await;

        match update_result {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }

    pub async fn follow_user_inner(
        conn: &Pool<Postgres>,
        follower_id: i64,
        following_id: i64
    ) -> Result<i64, sqlx::Error> {
        let id_result = sqlx
            ::query_as::<_, EntityId>(
                "insert into follow (follower_id, following_id) values ($1, $2) returning id"
            )
            .bind(follower_id)
            .bind(following_id)
            .fetch_one(conn).await;

        match id_result {
            Ok(row) => Ok(row.id),
            Err(e) => Err(e),
        }
    }

    pub async fn query_profile_inner(
        conn: &Pool<Postgres>,
        id: i64
    ) -> Result<Option<ProfileQueryResult>, sqlx::Error> {
        sqlx
            ::query_as::<_, ProfileQueryResult>("select * from profile where id = $1")
            .bind(id)
            .fetch_optional(conn).await
    }

    pub async fn query_profile_by_user_inner(
        conn: &Pool<Postgres>,
        user_name: String
    ) -> Result<Option<ProfileQueryResult>, sqlx::Error> {
        sqlx
            ::query_as::<_, ProfileQueryResult>("select * from profile where user_name = $1")
            .bind(user_name)
            .fetch_optional(conn).await
    }
}

#[automock]
#[async_trait]
pub trait InsertProfileFn {
    async fn insert_profile(
        &self,
        params: ProfileCreate
    ) -> Result<i64, sqlx::Error>;
}

#[async_trait]
impl InsertProfileFn for DbRepo {
    async fn insert_profile(
        &self,
        params: ProfileCreate
    ) -> Result<i64, sqlx::Error> {
        private_members::insert_profile_inner(self.get_conn(), params).await
    }
}

#[automock]
#[async_trait]
pub trait UpdateProfileAvatarFn {
    async fn update_profile_avatar(
        &self,
        user_id: i64,
        avatar: Vec<u8>
    ) -> Result<(), sqlx::Error>;
}

#[async_trait]
impl UpdateProfileAvatarFn for DbRepo {
    async fn update_profile_avatar(
        &self,
        user_id: i64,
        avatar: Vec<u8>
    ) -> Result<(), sqlx::Error> {
        private_members::update_profile_avatar_inner(self.get_conn(), user_id, avatar).await
    }
}

#[automock]
#[async_trait]
pub trait QueryProfileFn {
    async fn query_profile(
        &self,
        id: i64
    ) -> Result<Option<ProfileQueryResult>, sqlx::Error>;
}

#[async_trait]
impl QueryProfileFn for DbRepo {
    async fn query_profile(
        &self,
        id: i64
    ) -> Result<Option<ProfileQueryResult>, sqlx::Error> {
        private_members::query_profile_inner(self.get_conn(), id).await
    }
}

#[automock]
#[async_trait]
pub trait QueryProfileByUserFn {
    async fn query_profile_by_user(
        &self,
        user_name: String
    ) -> Result<Option<ProfileQueryResult>, sqlx::Error>;
}

#[async_trait]
impl QueryProfileByUserFn for DbRepo {
    async fn query_profile_by_user(
        &self,
        user_name: String
    ) -> Result<Option<ProfileQueryResult>, sqlx::Error> {
        private_members::query_profile_by_user_inner(self.get_conn(), user_name).await
    }
}

#[automock]
#[async_trait]
pub trait FollowUserFn {
    async fn follow_user(
        &self,
        follower_id: i64,
        following_id: i64
    ) -> Result<i64, sqlx::Error>;
}

#[async_trait]
impl FollowUserFn for DbRepo {
    async fn follow_user(
        &self,
        follower_id: i64,
        following_id: i64
    ) -> Result<i64, sqlx::Error> {
        private_members::follow_user_inner(self.get_conn(), follower_id, following_id).await
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        common_tests::actix_fixture::{
            PUBLIC_GROUP_TYPE,
            get_fake_message_body,
            FixtureError,
            MessageResponse,
        },
        common::entities::{
            messages::repo::{ InsertMessageFn, InsertResponseMessageFn },
            circle_group::repo::{ InsertCircleFn, InsertCircleMemberFn }, 
        },
    };
    use super::*;
    use fake::{ faker::lorem::en::Sentence, Fake };
    use lazy_static::lazy_static;
    use std::{ sync::{ Arc, RwLock }, collections::BTreeMap };
    use fake::faker::name::en::{ FirstName, LastName };
    use fake::faker::address::en::CountryName;
    use fake::faker::internet::en::Username;
    use rand::seq::SliceRandom;
    use std::ops::Range;

    #[derive(Clone)]
    #[allow(unused)]
    struct Fixtures {
        profiles: Vec<ProfileQueryResult>,
        db_repo: DbRepo
    }

    const PREFIX: &str = "Test profile";

    lazy_static! {
        static ref FIXTURES: Arc<RwLock<Option<Fixtures>>> = Arc::new(RwLock::new(None));
    }

    async fn setup_db_data(db_repo: DbRepo) -> Result<(), Box<dyn std::error::Error>> {
        let current_user = "current_user".to_string();
        let current_user_profile_result = db_repo.query_profile_by_user(
            current_user.clone()
        ).await;
        if let Err(_) = current_user_profile_result {
            return Err(
                Box::new(FixtureError::QueryFailed("Get current user profile failed".to_string()))
            );
        } else if let Ok(profile) = current_user_profile_result {
            if let Some(_) = profile {
                let message_reponse_result = sqlx
                    ::query_as::<_, MessageResponse>("select * from message_response")
                    .fetch_all(db_repo.get_conn()).await;
                match message_reponse_result {
                    Ok(row) => {
                        if row.len() > 0 {
                            println!("log: Necessary test data already set, exiting");
                            return Ok(());
                        }
                        return Err(
                            Box::new(
                                FixtureError::MissingData(
                                    "Message Response data missing".to_string()
                                )
                            )
                        );
                    }
                    Err(_) => {
                        return Err(
                            Box::new(
                                FixtureError::QueryFailed(
                                    "Message Response query failed".to_string()
                                )
                            )
                        );
                    }
                }
            }
        }

        println!("log: Need to setup test data");
        let tx = db_repo.get_conn().begin().await.unwrap();

        let description: String = Sentence(Range { start: 5, end: 8 }).fake();
        let current_profile_id = db_repo
            .insert_profile(ProfileCreate {
                user_name: current_user,
                full_name: "Current User".to_string(),
                description: format!("Test profile {} ", description),
                region: Some(CountryName().fake()),
                main_url: Some("http://current_user.com".to_string()),
                avatar: Some(vec![]),
            }).await
            .unwrap();

        let circle_group_id = db_repo.insert_circle(current_profile_id).await.unwrap();

        let mut following_profiles_and_messages: BTreeMap<i64, Vec<i64>> = BTreeMap::new();
        let local_prefix = PREFIX.clone();
        for _ in 1..11 {
            let first_name: String = FirstName().fake();
            let last_name: String = LastName().fake();
            let user_name: String = Username().fake();
            let following_profile_id = db_repo
                .insert_profile(ProfileCreate {
                    user_name: user_name.to_owned(),
                    full_name: format!("{} {}", first_name, last_name),
                    description: format!(
                        "{} {}",
                        local_prefix.clone(),
                        Sentence(Range { start: 5, end: 8 }).fake::<String>()
                    ),
                    region: Some("usa".to_string()),
                    main_url: Some(format!("http://{}.com", user_name)),
                    avatar: Some(vec![]),
                }).await
                .unwrap();

            _ = db_repo.follow_user(current_profile_id, following_profile_id).await;

            _ = db_repo.insert_circle_member(circle_group_id, following_profile_id).await;

            let mut following_profile_message_ids: Vec<i64> = vec![];
            for _ in 1..11 {
                let following_message_id = db_repo
                    .insert_message(
                        following_profile_id,
                        &get_fake_message_body(Some(local_prefix.clone().to_string())),
                        PUBLIC_GROUP_TYPE,
                        None
                    ).await
                    .unwrap();
                following_profile_message_ids.push(following_message_id);
            }
            following_profiles_and_messages.insert(
                following_profile_id,
                following_profile_message_ids
            );
        }
        println!("log: following_profiles_and_messages {:?}", following_profiles_and_messages);

        for following_pm in following_profiles_and_messages.iter() {
            let not_current_profile_ids: Vec<i64> = following_profiles_and_messages
                .clone()
                .into_iter()
                .filter(|pm_inner| { pm_inner.0.ne(following_pm.0) })
                .map(|pm_map| { pm_map.0 })
                .collect();

            for _ in 1..3 {
                let profile_id_to_broadcast = not_current_profile_ids
                    .choose(&mut rand::thread_rng())
                    .unwrap();
                // randomly select some messages of this profile
                // and have current profile create messages that broadcast the fhollowing user messages
                let broadcast_message_ids = following_profiles_and_messages
                    .get(&profile_id_to_broadcast)
                    .unwrap();
                for _ in [..4] {
                    let selected_message_id = broadcast_message_ids
                        .choose(&mut rand::thread_rng())
                        .unwrap();
                    _ = db_repo.insert_message(
                        *following_pm.0,
                        &get_fake_message_body(Some(local_prefix.clone().to_string())),
                        PUBLIC_GROUP_TYPE,
                        Some(*selected_message_id)
                    ).await;
                }
            }

            for _ in 1..4 {
                // select random profile
                let profile_id_to_respond_to = not_current_profile_ids
                    .choose(&mut rand::thread_rng())
                    .unwrap();

                let response_message_ids = following_profiles_and_messages
                    .get(&profile_id_to_respond_to)
                    .unwrap();
                for _ in [..4] {
                    let selected_message_id = response_message_ids
                        .choose(&mut rand::thread_rng())
                        .unwrap();
                    _ = db_repo.insert_response_message(
                        *following_pm.0,
                        &get_fake_message_body(Some(local_prefix.clone().to_string())),
                        PUBLIC_GROUP_TYPE,
                        *selected_message_id
                    ).await;
                }
            }
        }

        println!("log: Test data setup complete");
        _ = tx.commit().await;

        Ok(())
    }

    async fn setup_local_data(db_repo: DbRepo) -> Fixtures {
        setup_db_data(db_repo.clone()).await.unwrap();

        let profiles = sqlx
            ::query_as::<_, ProfileQueryResult>(
                "select * from profile where description like 'Test profile%'"
            )
            .fetch_all(db_repo.get_conn()).await
            .unwrap();

        Fixtures {
            profiles,
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

                *fx = Some(setup_local_data(db_repo).await);
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

    fn fixtures() -> Fixtures {
        Arc::clone(&FIXTURES).read().unwrap().clone().unwrap()
    }

    mod test_mod_insert_profile {
        use super::*;

        async fn test_insert_profile_body() {
            let fixtures = fixtures();

            let profile_id = fixtures.db_repo
                .insert_profile(ProfileCreate {
                    user_name: "user_a".to_string(),
                    full_name: "User A".to_string(),
                    description: "Test profile's description".to_string(),
                    region: Some("usa".to_string()),
                    main_url: Some("http://whatever.com".to_string()),
                    avatar: Some(vec![]),
                }).await
                .unwrap();

            assert!(profile_id > 0);
        }

        #[test]
        fn test_insert_profile() {
            RT.block_on(test_insert_profile_body())
        }
    }

    mod test_mod_update_profile_avatar {
        use crate::common_tests::actix_fixture::get_profile_avatar;

        use super::*;

        async fn test_update_profile_avatar_body() {
            let fixtures = fixtures();

            let mut mock_query_profile = MockQueryProfileFn::new();
            let profiles = fixtures.profiles.clone();
            mock_query_profile.expect_query_profile().returning(move |_| {
                Ok(
                    profiles
                        .clone()
                        .into_iter()
                        .find(|_| true)
                )
            });
            let profile = mock_query_profile
                .query_profile(0).await
                .unwrap()
                .unwrap();

            let profile_result = fixtures.db_repo.update_profile_avatar(
                profile.id,
                get_profile_avatar()
            ).await;

            assert!(profile_result.is_ok() == true);
        }

        #[test]
        fn test_update_profile_avatar() {
            RT.block_on(test_update_profile_avatar_body())
        }
    }

    mod test_mod_query_profile {
        use super::*;

        async fn test_insert_profile_and_get_profile_body() {
            let fixtures = fixtures();

            let selected_profile = fixtures.profiles[0].clone(); // arbitrarily get first profile
            let profile_to_create = ProfileCreate {
                user_name: selected_profile.user_name,
                full_name: selected_profile.full_name,
                description: selected_profile.description,
                region: selected_profile.region,
                main_url: selected_profile.main_url,
                avatar: selected_profile.avatar,
            };
            let mut mock_repo = MockInsertProfileFn::new();
            let user_name = profile_to_create.user_name.clone();
            mock_repo
                .expect_insert_profile()
                .withf(move |params| params.user_name == user_name)
                .times(1)
                .returning(move |params| {
                    Ok(
                        fixtures.profiles
                            .iter()
                            .find(|p| { p.user_name == params.user_name })
                            .unwrap().id
                    )
                });

            let profile_id = mock_repo
                .insert_profile(profile_to_create.clone()).await
                .unwrap();

            let profile = fixtures.db_repo.query_profile(profile_id).await.unwrap().unwrap();

            assert!(profile_id > 0);
            assert!(profile.id == profile_id);
            assert!(profile.user_name == profile_to_create.user_name);
            assert!(profile.full_name == profile_to_create.full_name);
            assert!(profile.description == profile_to_create.description);
            assert!(profile.region == profile_to_create.region);
            assert!(profile.main_url == profile_to_create.main_url);
        }

        #[test]
        fn test_insert_profile_and_get_profile() {
            RT.block_on(test_insert_profile_and_get_profile_body())
        }
    }

    mod test_mod_query_profile_by_user {
        use super::*;

        async fn test_insert_profile_and_get_profile_by_user_body() {
            let fixtures = fixtures();

            let selected_profile = fixtures.profiles[0].clone();
            let profile_to_create = ProfileCreate {
                user_name: selected_profile.user_name.clone(),
                full_name: selected_profile.full_name,
                description: selected_profile.description,
                region: selected_profile.region,
                main_url: selected_profile.main_url,
                avatar: selected_profile.avatar,
            };
            let mut mock_insert_repo = MockInsertProfileFn::new();
            let profiles = fixtures.profiles.clone();
            mock_insert_repo
                .expect_insert_profile()
                .times(1)
                .returning(move |params| {
                    Ok(
                        profiles
                            .clone()
                            .iter()
                            .find(|p| { p.user_name == params.user_name })
                            .unwrap().id
                    )
                });
            let profile_id = mock_insert_repo
                .insert_profile(profile_to_create.clone()).await
                .unwrap();

            let profile = fixtures.db_repo
                .query_profile_by_user(selected_profile.user_name).await
                .unwrap()
                .unwrap();

            assert!(profile_id > 0);
            assert!(profile.id == profile_id);
            assert!(profile.user_name == profile_to_create.user_name);
            assert!(profile.full_name == profile_to_create.full_name);
            assert!(profile.description == profile_to_create.description);
            assert!(profile.region == profile_to_create.region);
            assert!(profile.main_url == profile_to_create.main_url);
        }

        #[test]
        fn test_insert_profile_and_get_profile_by_user() {
            RT.block_on(test_insert_profile_and_get_profile_by_user_body())
        }
    }

    mod test_mod_insert_follower {
        use super::*;

        async fn test_insert_follow_user_body() {
            let fixtures = fixtures();

            let mut mock_insert_repo = MockInsertProfileFn::new();
            let profiles = fixtures.profiles.clone();
            mock_insert_repo
                .expect_insert_profile()
                .times(2)
                .returning(move |params| {
                    Ok(
                        profiles
                            .clone()
                            .into_iter()
                            .find(|p| { p.user_name == params.user_name })
                            .unwrap().id
                    )
                });

            let selected_follower_profile = fixtures.profiles[0].clone();
            let follower_id = mock_insert_repo
                .insert_profile(ProfileCreate {
                    user_name: selected_follower_profile.user_name,
                    full_name: selected_follower_profile.full_name,
                    description: selected_follower_profile.description,
                    region: selected_follower_profile.region,
                    main_url: selected_follower_profile.main_url,
                    avatar: selected_follower_profile.avatar,
                }).await
                .unwrap();

            let selected_following_profile = fixtures.profiles[1].clone();
            let following_id = mock_insert_repo
                .insert_profile(ProfileCreate {
                    user_name: selected_following_profile.user_name,
                    full_name: selected_following_profile.full_name,
                    description: selected_following_profile.description,
                    region: selected_following_profile.region,
                    main_url: selected_following_profile.main_url,
                    avatar: selected_following_profile.avatar,
                }).await
                .unwrap();

            let follow_id = fixtures.db_repo
                .follow_user(follower_id, following_id).await
                .unwrap();

            assert!(follow_id > 0);
        }

        #[test]
        fn test_insert_follow_user() {
            RT.block_on(test_insert_follow_user_body())
        }
    }
}

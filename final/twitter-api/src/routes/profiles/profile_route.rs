use crate::{common::{
    app_state::AppState,
    entities::{
        profiles::{
            model::{ ProfileCreate, ProfileQueryResult },
            repo::{ InsertProfileFn, QueryProfileFn, QueryProfileByUserFn },
        },
    },
}, routes::{errors::error_utils::UserError, output_id::OutputId}};
use actix_web::{ web, web::Path };
use log::info;
use super::model::{
    ProfileQuery,
    ProfileByUserNameQuery,
    ProfileResponder,
    ProfileCreateMultipart,
};

#[allow(unused)]
pub async fn create_profile<T: InsertProfileFn>(
    app_data: web::Data<AppState<T>>,
    form: ProfileCreateMultipart
) -> Result<OutputId, UserError> {
    let result = app_data.db_repo.insert_profile(ProfileCreate {
        user_name: form.user_name.to_owned(),
        full_name: form.full_name.to_owned(),
        description: form.description.to_owned(),
        region: match form.region.as_ref() {
            Some(region) => Some(region.to_owned()),
            None => None,
        },
        main_url: match form.main_url.as_ref() {
            Some(main_url) => Some(main_url.to_owned()),
            None => None,
        },
        avatar: form.avatar,
    }).await;

    match result {
        Ok(id) => Ok(OutputId { id }),
        Err(e) => Err(e.into()),
    }
}

pub async fn get_profile<T: QueryProfileFn>(
    app_data: web::Data<AppState<T>>,
    path: Path<ProfileQuery>
) -> Result<Option<ProfileResponder>, UserError> {
    info!("start get_profile");
    let result = app_data.db_repo.query_profile(path.id).await;

    match result {
        Ok(profile) => Ok(convert(profile)),
        Err(e) => Err(e.into())
    }
}

pub async fn get_profile_by_user<T: QueryProfileByUserFn>(
    app_data: web::Data<AppState<T>>,
    path: Path<ProfileByUserNameQuery>
) -> Result<Option<ProfileResponder>, UserError> {
    let result = app_data.db_repo.query_profile_by_user(
        path.user_name.to_owned()
    ).await;

    match result {
        Ok(profile) => Ok(convert(profile)),
        Err(e) => Err(e.into()),
    }
}

fn convert(profile: Option<ProfileQueryResult>) -> Option<ProfileResponder> {
    match profile {
        Some(item) =>
            Some(ProfileResponder {
                id: item.id,
                created_at: item.created_at,
                user_name: item.user_name,
                full_name: item.full_name,
                description: item.description,
                region: item.region,
                main_url: item.main_url,
                avatar: item.avatar,
            }),
        None => None,
    }
}


#[cfg(test)]
mod tests {
    use fake::{
        faker::{
            internet::en::Username, 
            name::en::{LastName, FirstName}, 
            lorem::en::Sentence, address::en::CountryName
        }, 
        Fake
    };
    use crate::{
        common::{
            entities::{profiles::{repo::InsertProfileFn, model::ProfileCreate}}
        }, 
        common_tests::actix_fixture::{get_profile_avatar, get_fake_main_url, get_app_data
        }, routes::{profiles::model::ProfileCreateMultipart, errors::error_utils::UserError}
    };
    use super::*;
    use async_trait::async_trait;
    use actix_web::web::Path;

    mod test_mod_create_profile_failure_returns_correct_error {    
        use super::*;

        #[derive(Clone)]
        struct MockDbRepo;

        #[async_trait]
        impl InsertProfileFn for MockDbRepo {
            async fn insert_profile(&self, _: ProfileCreate) -> Result<i64, sqlx::Error> {
                Err(sqlx::Error::RowNotFound)
            }
        }

        #[tokio::test]
        async fn test_create_profile_failure_returns_correct_error() {          
            let avatar = get_profile_avatar();

            let app_data = get_app_data(MockDbRepo).await;

            let result = create_profile(app_data, ProfileCreateMultipart { 
                user_name: Username().fake::<String>(), 
                full_name: format!("{} {}", FirstName().fake::<String>(), LastName().fake::<String>()),
                description: Sentence(1..2).fake::<String>(), 
                region: Some(CountryName().fake::<String>()), 
                main_url: Some(get_fake_main_url()),
                avatar: Some(avatar), 
            }).await;
            
            assert!(result.is_err() == true);
            assert!(result.err().unwrap() == UserError::InternalError);
        }
    }

    mod test_mod_create_profile_and_check_id {    
        use super::*;

        const ID: i64 = 22;

        #[derive(Clone)]
        struct MockDbRepo;

        #[async_trait]
        impl InsertProfileFn for MockDbRepo {
            async fn insert_profile(&self, _: ProfileCreate) -> Result<i64, sqlx::Error> {
                Ok(ID)
            }
        }

        #[tokio::test]
        async fn test_create_profile_and_check_id() {          
            let avatar = get_profile_avatar();

            let app_data = get_app_data(MockDbRepo).await;

            let result = create_profile(app_data, ProfileCreateMultipart { 
                user_name: Username().fake::<String>(), 
                full_name: format!("{} {}", FirstName().fake::<String>(), LastName().fake::<String>()),
                description: Sentence(1..2).fake::<String>(), 
                region: Some(CountryName().fake::<String>()), 
                main_url: Some(get_fake_main_url()),
                avatar: Some(avatar), 
            }).await;
            
            assert!(!result.is_err());
            assert!(result.ok().unwrap().id == ID);
        }
    }

    mod test_mod_get_profile_failure_returns_correct_error {    
        use super::*;
        
        #[derive(Clone)]
        struct MockDbRepo;

        #[async_trait]
        impl QueryProfileFn for MockDbRepo {
            async fn query_profile(&self, _: i64) -> Result<Option<ProfileQueryResult>, sqlx::Error> {
                Err(sqlx::Error::RowNotFound)
            }
        }

        #[tokio::test]
        async fn test_get_profile_failure_returns_correct_error() {
            let app_data = get_app_data(MockDbRepo).await;

            let get_result = get_profile(app_data, Path::from(ProfileQuery { id: 0 })).await;

            assert!(get_result.is_err() == true);
            assert!(get_result.err().unwrap() == UserError::InternalError);
        }
    }

    mod test_mod_get_profile_and_check_id {    
        use chrono::Utc;
        use crate::common_tests::actix_fixture::get_fake_message_body;
        use super::*;
        
        const ID: i64 = 22;
        #[derive(Clone)]
        struct MockDbRepo;

        #[async_trait]
        impl QueryProfileFn for MockDbRepo {
            async fn query_profile(&self, _: i64) -> Result<Option<ProfileQueryResult>, sqlx::Error> {
                Ok(Some(ProfileQueryResult {
                    id: ID,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    user_name: Username().fake(),
                    full_name: format!("{} {} ", FirstName().fake::<String>(), LastName().fake::<String>()),
                    description: get_fake_message_body(None),
                    region: None,
                    main_url: None,
                    avatar: None
                }))
            }
        }

        #[tokio::test]
        async fn test_get_profile_and_check_id() {
            let app_data = get_app_data(MockDbRepo).await;

            let get_result = get_profile(app_data, Path::from(ProfileQuery { id: 0 })).await;

            assert!(!get_result.is_err());
            assert!(get_result.ok().unwrap().unwrap().id == ID);
        }
    }

    mod test_mod_get_profile_by_user_failure_returns_correct_error {    
        use super::*;
        
        #[derive(Clone)]
        struct MockDbRepo;

        #[async_trait]
        impl QueryProfileByUserFn for MockDbRepo {
            async fn query_profile_by_user(&self, _: String) -> Result<Option<ProfileQueryResult>, sqlx::Error> {
                Err(sqlx::Error::RowNotFound)
            }
        }

        #[tokio::test]
        async fn test_get_profile_by_user_failure_returns_correct_error() {
            let app_data = get_app_data(MockDbRepo).await;

            let get_result = get_profile_by_user(app_data, Path::from(ProfileByUserNameQuery { user_name: Username().fake::<String>() })).await;

            assert!(get_result.as_ref().is_err() == true);
            assert!(get_result.err().unwrap() == UserError::InternalError);
        }
    }

    mod test_mod_get_profile_by_user_and_check_id {    
        use chrono::Utc;
        use crate::common_tests::actix_fixture::get_fake_message_body;
        use super::*;
        
        const ID: i64 = 22;
        #[derive(Clone)]
        struct MockDbRepo;

        #[async_trait]
        impl QueryProfileByUserFn for MockDbRepo {
            async fn query_profile_by_user(&self, _: String) -> Result<Option<ProfileQueryResult>, sqlx::Error> {
                Ok(Some(ProfileQueryResult {
                    id: ID,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    user_name: Username().fake(),
                    full_name: format!("{} {} ", FirstName().fake::<String>(), LastName().fake::<String>()),
                    description: get_fake_message_body(None),
                    region: None,
                    main_url: None,
                    avatar: None
                }))
            }
        }

        #[tokio::test]
        async fn test_get_profile_by_user_and_check_id() {
            let app_data = get_app_data(MockDbRepo).await;

            let get_result = get_profile_by_user(app_data, Path::from(ProfileByUserNameQuery { user_name: Username().fake() })).await;

            assert!(!get_result.is_err());
            assert!(get_result.ok().unwrap().unwrap().id == ID);
        }
    }
}
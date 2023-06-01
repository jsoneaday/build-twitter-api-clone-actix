use crate::{common::{
    app_state::AppState,
    entities::{
        profiles::{
            model::{ ProfileCreate, ProfileQueryResult },
            repo::{ InsertProfileFn, QueryProfileFn, QueryProfileByUserFn },
        },
    },
}, routes::errors::error_utils::UserError};
use actix_web::{ web, get, web::{ Path, Json }, Responder };
use super::model::{
    ProfileQuery,
    ProfileByUserNameQuery,
    ProfileResponder,
    ProfileCreateMultipart,
};

#[allow(unused)]
pub async fn create_profile(
    app_data: web::Data<AppState>,
    form: ProfileCreateMultipart
) -> Result<impl Responder, UserError> {
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
        Ok(id) => Ok(Json(id)),
        Err(e) => Err(e.into()),
    }
}

#[get("/profile/{id}")]
pub async fn get_profile(
    app_data: web::Data<AppState>,
    path: Path<ProfileQuery>
) -> Result<impl Responder, UserError> {
    let result = app_data.db_repo.query_profile(path.id).await;

    match result {
        Ok(profile) => { Ok(Json(convert(profile))) }
        Err(e) => Err(e.into()),
    }
}

#[get("/profile/username/{user_name}")]
pub async fn get_profile_by_user(
    app_data: web::Data<AppState>,
    path: Path<ProfileByUserNameQuery>
) -> Result<impl Responder, UserError> {
    let result = app_data.db_repo.query_profile_by_user(
        path.user_name.to_owned()
    ).await;

    match result {
        Ok(profile) => Ok(Json(convert(profile))),
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

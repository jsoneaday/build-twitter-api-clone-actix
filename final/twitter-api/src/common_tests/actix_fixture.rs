use crate::{
    common::{ app_state::AppState, fs::file_utils::get_avatar_buffer, entities::{base::DbRepo}},
    routes::{
        //messages::message_route::{ get_message, create_message, get_messages },
        profiles::{ profile_route::{ create_profile, get_profile, get_profile_by_user } }, messages::message_route::create_message,
    },
};
use chrono::{ DateTime, Utc };
use serde::Deserialize;
use sqlx::{ FromRow };
use actix_web::{ App, web::{ self, BytesMut, Bytes }, Error, test, dev::{ Service, ServiceResponse } };
use actix_http::Request;
use fake::{
    Fake,
    faker::{
        internet::{ en::Username, en::DomainSuffix },
        name::en::{ FirstName, LastName },
        address::en::CountryName,
    },
};
use fake::faker::lorem::en::Sentence;
use fake::faker::company::en::CompanyName;
use std::{ ops::Range };

pub const PUBLIC_GROUP_TYPE: i32 = 1;
pub const CIRCLE_GROUP_TYPE: i32 = 2;

#[allow(unused)]
#[derive(Deserialize, FromRow)]
pub struct MessageResponse {
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub original_msg_id: i64,
    pub responding_msg_id: i64,
}

#[derive(Debug, Clone)]
pub enum FixtureError {
    MissingData(String),
    QueryFailed(String),
}
impl std::error::Error for FixtureError {}
impl std::fmt::Display for FixtureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingData(msg) => write!(f, "{}", msg),
            Self::QueryFailed(msg) => write!(f, "{}", msg),
        }
    }
}

#[allow(unused)]
pub async fn get_app_state<T>(db_repo: T) -> AppState<T> {
    AppState {
        client: reqwest::Client::new(),
        db_repo,
    }
}

pub async fn get_app_data<T>(db_repo: T) -> web::Data<AppState<T>> {
    web::Data::new(get_app_state(db_repo).await)
}

#[allow(unused)]
pub async fn get_app() -> impl Service<Request, Response = ServiceResponse, Error = Error> {
    // std::env::set_var("RUST_LOG", "debug");
    // env_logger::init();

    let app_data = get_app_data(DbRepo::init().await).await;
    test::init_service(
        App::new()
            .app_data(app_data.clone())            
            .service(
                web
                    ::scope("/v1")
                    .service(
                        web
                            ::resource("/msg")
                            // .route(web::get().to(get_message))
                            .route(web::post().to(create_message::<DbRepo>))
                    )
                    //.service(web::resource("/msgs").route(web::get().to(get_messages)))
                    .service(web::resource("/profile/{id}").route(web::get().to(get_profile::<DbRepo>)))
                    .service(web::resource("/profile/username/{user_name}").route(web::get().to(get_profile_by_user::<DbRepo>)))
                    .service(web::resource("/profile").route(web::post().to(create_profile::<DbRepo>)))
            )
    ).await
}

pub fn create_random_msg_body(prefix: Option<String>) -> String {
    let mut body: String = match prefix {
        Some(pref) => pref,
        None => "".to_string(),
    };

    for _ in [..4] {
        let random_sentence: String = Sentence(Range { start: 5, end: 6 }).fake();
        body = format!("{}. {}", body, random_sentence);
    }
    body
}

/// warning: line breaks are very important when ending any line!!!
pub fn get_profile_create_multipart(
    avatar: &Vec<u8>,
    boundary: &str,
    with_avatar: bool
) -> BytesMut {
    let mut payload = actix_web::web::BytesMut::new();
    payload.extend(format!("--{}\r\n", boundary).as_bytes());
    payload.extend(
        format!("Content-Disposition: form-data; name=\"user_name\"\r\n\r\n").as_bytes()
    );
    payload.extend(format!("{}\r\n", Username().fake::<String>()).as_bytes());
    payload.extend(format!("--{}\r\n", boundary).as_bytes());
    payload.extend(
        format!("Content-Disposition: form-data; name=\"full_name\"\r\n\r\n").as_bytes()
    );
    payload.extend(
        format!("{} {}\r\n", FirstName().fake::<String>(), LastName().fake::<String>()).as_bytes()
    );
    payload.extend(format!("--{}\r\n", boundary).as_bytes());
    payload.extend(
        format!("Content-Disposition: form-data; name=\"description\"\r\n\r\n").as_bytes()
    );
    payload.extend(
        format!("{}\r\n", Sentence(Range { start: 8, end: 10 }).fake::<String>()).as_bytes()
    );
    payload.extend(format!("--{}\r\n", boundary).as_bytes());
    payload.extend(format!("Content-Disposition: form-data; name=\"region\"\r\n\r\n").as_bytes());
    payload.extend(format!("{}\r\n", CountryName().fake::<String>()).as_bytes());
    payload.extend(format!("--{}\r\n", boundary).as_bytes());
    payload.extend(format!("Content-Disposition: form-data; name=\"main_url\"\r\n\r\n").as_bytes());
    let mut domain = CompanyName().fake::<String>();
    domain.retain(|str| !str.is_whitespace());
    payload.extend(get_fake_main_url().as_bytes());
    payload.extend(format!("--{}\r\n", boundary).as_bytes());

    if with_avatar == true {
        payload.extend(
            format!(
                "Content-Disposition: form-data; name=\"avatar\"; filename=\"profile.jpeg\"\r\n\
             Content-Type: image/jpeg\r\n\r\n"
            ).as_bytes()
        );
        payload.extend(Bytes::from(avatar.clone()));
        payload.extend(b"\r\n"); // warning: line breaks are very important!!!
        payload.extend(format!("--{}--\r\n", boundary).as_bytes()); // note the extra -- at the end
    }

    payload
}

pub fn get_fake_main_url() -> String {
    let mut domain = CompanyName().fake::<String>();
    domain.retain(|str| !str.is_whitespace());
    format!("https://{}.{}", domain, DomainSuffix().fake::<String>())
}

pub fn get_profile_avatar() -> Vec<u8> {
    let file_name = "profile.jpeg".to_string();
    let file_path = format!("src/common_tests/{}", file_name);

    get_avatar_buffer(&file_path)
}

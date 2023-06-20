use crate::{
    common::{ app_state::AppState, fs::file_utils::get_avatar_buffer, entities::{base::DbRepo}},
    routes::{
        profiles::{ profile_route::{ create_profile, get_profile, get_profile_by_user } }, messages::message_route::{create_message, get_message, get_messages},
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
const JPEG_SIGNATURE: [u8; 2] = [0xFF, 0xD8];
#[allow(unused)]
const JPEG_END_SIGNATURE: [u8; 2] = [0xFF, 0xD9];

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
fn is_jpeg(avatar: Vec<u8>) -> bool {
    let mut is_valid = false;
    if avatar.len() >= 2 && &avatar[0..2] == JPEG_SIGNATURE {
        let end_offset = avatar.len() - 2;
        if &avatar[end_offset..] == JPEG_END_SIGNATURE {
            println!("The avatar data is a valid JPEG image.");
            is_valid = true;
        } else {
            println!("The avatar data does not have a valid JPEG end signature.");
        }
    } else {
        println!("The avatar data does not have a valid JPEG signature.");
    }
    is_valid
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
    let app_data = get_app_data(DbRepo::init().await).await;
    test::init_service(
        App::new()
            .app_data(app_data.clone())            
            .service(
                web::scope("/v1")
                    .service(web::resource("/msg/{id}").route(web::get().to(get_message::<DbRepo>)))
                    .service(web::resource("/msg").route(web::post().to(create_message::<DbRepo>)))
                    .service(web::resource("/msgs").route(web::post().to(get_messages::<DbRepo>)))
                    .service(web::resource("/profile/{id}").route(web::get().to(get_profile::<DbRepo>)))
                    .service(web::resource("/profile/username/{user_name}").route(web::get().to(get_profile_by_user::<DbRepo>)))
                    .service(web::resource("/profile").route(web::post().to(create_profile::<DbRepo>)))
            )
    ).await
}

pub fn get_fake_message_body(prefix: Option<String>) -> String {
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
    payload.extend(get_fake_main_url().as_bytes());
    payload.extend(b"\r\n"); // warning: line breaks are very important!!! 
    payload.extend(format!("--{}\r\n", boundary).as_bytes());

    if with_avatar == true {
        payload.extend(
            b"Content-Disposition: form-data; name=\"avatar\"; filename=\"profile.jpeg\"\r\n"
        );
        payload.extend(b"Content-Type: image/jpeg\r\n\r\n");
        payload.extend(Bytes::from(avatar.clone()));
        payload.extend(b"\r\n"); // warning: line breaks are very important!!!        
    }
    payload.extend(format!("--{}--\r\n", boundary).as_bytes()); // note the extra -- at the end of the boundary

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

use actix_web::{web::Json, Responder};
use serde::Serialize;

#[derive(Serialize)]
struct Person {
    name: String
}

pub async fn get_profile_name() -> impl Responder {
    Json(Person { name: "lynn".to_string() })
}
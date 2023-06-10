use serde::Serialize;
use actix_web::{Responder, body::BoxBody, HttpResponse, http::header::ContentType};

#[derive(Serialize)]
pub struct Person {
    pub name: String
}

impl Responder for Person {
    type Body = BoxBody;

    fn respond_to(self, _: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        let body_result = serde_json::to_string(&self);

        match body_result {
            Ok(body) => {
                HttpResponse::Ok()
                .insert_header(("test", "test"))
                .content_type(ContentType::json())
                .body(body)
            },
            Err(_) => {
                HttpResponse::InternalServerError()
                    .content_type(ContentType::plaintext())
                    .body("Failure!")
            },
        }
    }
}

pub async fn get_profile_name() -> Person {
    Person {
        name: "adam".to_string()
    }
}
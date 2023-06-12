use actix_http::body::BoxBody;
use actix_web::http::header::ContentType;
use actix_web::{Responder, HttpResponse, HttpRequest };
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct OutputId {
    pub id: i64
}

impl Responder for OutputId {
    type Body = BoxBody;

    fn respond_to(self, _: &HttpRequest) -> HttpResponse<Self::Body> {
        let body_result = serde_json::to_string(&self);

        match body_result {
            Ok(body) => {
                HttpResponse::Ok()
                .content_type(ContentType::json())
                .body(body)
            },
            Err(_) => {
                HttpResponse::InternalServerError()
                    .content_type(ContentType::json())
                    .body("Failed to serialize id.")
            },
        }
    }
}
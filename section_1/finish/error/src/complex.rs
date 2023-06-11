
use actix_web::{ResponseError, HttpResponse, Result, http::header::ContentType};
use derive_more::{Display, Error};

#[allow(unused)]
#[derive(Debug, Display, Error)]
pub enum MyError {
    #[display(fmt = "Internal server error")]
    InternalError,
    #[display(fmt = "A field value is invalid {}", field)]
    ValidationError { field: String },
    #[display(fmt = "An unknown error has occured")]
    UnknownError
}

impl ResponseError for MyError {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code())
            .content_type(ContentType::plaintext())
            .body(self.to_string())
    }

    fn status_code(&self) -> actix_web::http::StatusCode {
        match *self {
            MyError::InternalError => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            MyError::ValidationError { .. } => actix_web::http::StatusCode::BAD_REQUEST,
            MyError::UnknownError => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub async fn get() -> Result<String, actix_web::error::Error> {    
    Err(MyError::ValidationError { field: "user_name".to_string() }.into())
}
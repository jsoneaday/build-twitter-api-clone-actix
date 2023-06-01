use actix_web::{ 
    http::{ header::ContentType, StatusCode },
    HttpResponse, 
    ResponseError 
};
use derive_more::{Display, Error};

/// errors visible by the user
#[derive(Debug, Display, Error)]
pub enum UserErrors {
    #[display(fmt = "An internal error occurred. Please try again later.")]
    InternalError,
    #[display(fmt = "Validation error on field: {}", field)]
    ValidationError { field: String },
}

impl ResponseError for UserErrors {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            UserErrors::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            UserErrors::ValidationError { .. } => StatusCode::BAD_REQUEST,
        }
    }
}
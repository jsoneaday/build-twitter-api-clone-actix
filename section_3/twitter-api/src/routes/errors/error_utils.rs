use actix_web::{ 
    http::{ header::ContentType, StatusCode },
    HttpResponse, 
    ResponseError 
};
use derive_more::{Display, Error};

/// errors visible by the user
#[derive(Debug, Display, Error, PartialEq)]
pub enum UserError {
    #[display(fmt = "An internal error occurred. Please try again later.")]
    InternalError,
    #[display(fmt = "Validation error on field: {}", field)]
    ValidationError { field: String },
}

impl UserError {
    pub fn convert_to_user_error(e: sqlx::Error) -> UserError {
        match e {
            sqlx::Error::RowNotFound => UserError::InternalError,
            sqlx::Error::ColumnDecode { .. } => UserError::InternalError,
            sqlx::Error::Decode(_) => UserError::InternalError,
            sqlx::Error::PoolTimedOut => UserError::InternalError,
            sqlx::Error::PoolClosed => UserError::InternalError,
            sqlx::Error::WorkerCrashed => UserError::InternalError,
            #[cfg(feature = "migrate")]
            sqlx::Error::Migrate(_) => UserError::InternalError,
            _ => UserError::InternalError,
        }
    }
}

impl ResponseError for UserError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            UserError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            UserError::ValidationError { .. } => StatusCode::BAD_REQUEST,
        }
    }
}

impl Into<UserError> for sqlx::Error {
    fn into(self) -> UserError {
        UserError::convert_to_user_error(self)
    }
}
use actix_web::{error, Result};

pub struct UserError {
    pub message: &'static str
}

pub async fn get() -> Result<String> {
    let err = Err(UserError { message: "internal danger" });

    err.map_err(|err| error::ErrorInternalServerError(err.message))
}
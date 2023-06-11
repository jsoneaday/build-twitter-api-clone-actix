use actix_web::{error, Result};

pub struct UserError {
    pub name: &'static str
}

pub async fn get() -> Result<String> {
    let err = Err(UserError { name: "internal danger" });

    err.map_err(|err| error::ErrorInternalServerError(err.name))
}
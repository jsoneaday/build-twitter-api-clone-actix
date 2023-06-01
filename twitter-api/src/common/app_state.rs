use super::entities::base::{DbRepo};

#[derive(Clone)]
pub struct AppState {
    pub client: reqwest::Client,
    pub db_repo: DbRepo,
}
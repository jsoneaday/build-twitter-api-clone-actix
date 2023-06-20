pub struct AppState<T> {
    pub client: reqwest::Client,
    pub db_repo: T,
}

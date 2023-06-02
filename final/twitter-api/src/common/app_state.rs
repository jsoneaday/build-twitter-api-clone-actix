pub struct AppState<T: ?Sized> {
    pub client: reqwest::Client,
    pub db_repo: T,
}

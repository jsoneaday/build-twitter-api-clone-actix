use twitter_clone_api::run;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    run().await
}

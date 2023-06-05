use actix_web::HttpServer;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        actix_web::App::new()
            .service(
                actix_web::web::scope("/v1")
                    .route("/profile", actix_web::web::get().to(index))
                    .route("/profile", actix_web::web::post().to(insert))
            )
    })
    .bind(("127.0.0.1", 8001))?
    .run()
    .await
}

async fn index() -> &'static str {
    "hello world"
}

async fn insert() -> &'static str {
    "inserted"
}
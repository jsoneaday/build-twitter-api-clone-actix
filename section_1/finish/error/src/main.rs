mod complex;
use crate::complex::{get};
use actix_web::{HttpServer, App, web};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
        .route("/", web::get().to(get))    
    })
    .bind(("127.0.0.1", 8001))?
    .run()
    .await
}


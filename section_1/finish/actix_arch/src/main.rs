use actix_web::{HttpServer, web};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        actix_web::App::new()
            .route("/", web::get().to(index))
            .service(
                web::scope("/v1")
                    .route("/item", web::get().to(get_item)) // append additional routes 
            ) // append additional services
    })
        .bind(("127.0.0.1", 8001))?
        .run()
        .await
}

async fn index() -> &'static str {
    "Hello World!"
}

async fn get_item() -> &'static str {
    "Item A"
}
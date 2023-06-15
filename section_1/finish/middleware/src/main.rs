mod req_middleware;

use actix_web::{
    HttpServer, 
    App, 
    web::post,
};
use crate::req_middleware::{do_it, AppenderMiddlewareBuilder};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .wrap(AppenderMiddlewareBuilder)
            .route("/", post().to(do_it))
    })
    .bind(("127.0.0.1", 8001))?
    .run()
    .await
}

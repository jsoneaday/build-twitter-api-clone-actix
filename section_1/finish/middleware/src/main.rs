mod req_res_middleware;

use actix_web::{
    HttpServer, 
    App, 
    web::post,
    middleware::Logger,
};
use crate::req_res_middleware::{do_it, ReqAppenderMiddlewareBuilder};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(ReqAppenderMiddlewareBuilder)            
            .route("/", post().to(do_it))
    })
    .bind(("127.0.0.1", 8001))?
    .run()
    .await
}

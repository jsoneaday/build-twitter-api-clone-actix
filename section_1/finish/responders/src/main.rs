mod either_response;

use crate::either_response::get_profile_name;
use actix_web::{HttpServer, App, web};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .service(
                web::resource("/profile/{side}")
                .route(web::get().to(get_profile_name))   
            )
    })
    .bind(("127.0.0.1", 8001))?
    .run()
    .await
}
pub mod common {
    pub mod app_state;
    pub mod entities {
        pub mod messages {
            pub mod model;
            pub mod repo;
        }
        pub mod profiles {
            pub mod model;
            pub mod repo;
        }
        pub mod circle_group {
            pub mod model;
            pub mod repo;
        }
        pub mod base;
    }
    pub mod fs {
        pub mod file_utils;
    }    
}
pub mod common_tests {
    pub mod actix_fixture;
}
pub mod routes {
    pub mod messages {
        pub mod model;
        pub mod message_route;
    }
    pub mod profiles {
        pub mod model;
        pub mod profile_route;
    }
    pub mod errors {
        pub mod error_utils;
    }
}

use std::env;
use common::entities::{base::DbRepo, messages::repo::InsertMessageFn};
use dotenv::dotenv;
use actix_web::{ web, App, HttpServer, Responder };
use routes::profiles::profile_route::{ create_profile, get_profile, get_profile_by_user };
use std::error::Error;
use crate::common::app_state::AppState;
use crate::routes::messages::message_route::{ create_message };

pub async fn run() -> std::io::Result<()> {
    dotenv().ok();
    let port = env::var("PORT").unwrap().parse().unwrap();
    let host = env::var("HOST").unwrap();
    let db_repo = DbRepo::init().await;
    // HttpServer runs in multiple worker threads usually one for each cpu core
    // Each server therefore gets its own instance of AppState    
    // therefore creating the app_data here outside of the HttpServer and cloning it for each App instance is safer and prevents syncing issues if data changes later
    let app_data = web::Data::new(AppState {
                    client: reqwest::Client::new(),
                    db_repo,
                });
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let result = HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .route("/", web::get().to(get_root))
            .service(
                web
                    ::scope("/v1")
                    .service(
                        web
                            ::resource("/msg")
                            // .route(web::get().to(get_message))
                            .route(web::post().to(create_message::<dyn InsertMessageFn>))
                    )
                    //.service(web::resource("/msgs").route(web::get().to(get_messages)))
                    .service(web::resource("/profile/{id}").route(web::get().to(get_profile::<DbRepo>)))
                    .service(web::resource("/profile/username/{user_name}").route(web::get().to(get_profile_by_user::<DbRepo>)))
                    .service(web::resource("/profile").route(web::post().to(create_profile::<DbRepo>)))
            )
    })
    .bind((host, port))?
    .run().await;

    result
}

#[allow(unused)]
pub async fn get_root() -> Result<impl Responder, Box<dyn Error>> {
    Ok("Hello World!!!")
}

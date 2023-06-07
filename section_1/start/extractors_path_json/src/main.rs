use std::sync::Mutex;

use actix_web::{HttpServer, App, web, web::Data};

struct Messenger {
    message: String
}

struct MutableState {
    messenger: Mutex<Messenger>
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let data = actix_web::web::Data::new(MutableState {
        messenger: Mutex::new(Messenger { message: "hello".to_string() })
    });

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .route("/", web::post().to(insert))
            .route("/", web::get().to(get))
    })
    .bind(("127.0.0.1", 8001))?
    .run()
    .await
}

async fn insert(app_data: Data<MutableState>) -> String {
    let mut messenger = app_data.messenger.lock().unwrap();
    messenger.message = format!("{} world", messenger.message);
    "".to_string()
}

async fn get(app_data: Data<MutableState>) -> String {
    app_data.messenger.lock().unwrap().message.clone()
}
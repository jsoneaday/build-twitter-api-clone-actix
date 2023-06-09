use actix_web::{HttpServer, App, web::{self, Path, Json}};
use serde::{Deserialize};

#[allow(unused)]
#[derive(Clone)]
struct FinalUser {
    id: i64,
    user_name: String,
    full_name: String
}

#[derive(Deserialize)]
struct NewUser {
    user_name: String,
    full_name: String
}

#[derive(Deserialize)]
struct EntityId {
    id: i64
}

struct AppState {
    users: std::sync::RwLock<Vec<FinalUser>>
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_data = web::Data::new(AppState {
        users: std::sync::RwLock::new(vec![
            FinalUser { id: 1, user_name: "dave".to_string(), full_name: "Dave Choi".to_string() },
            FinalUser { id: 2, user_name: "cindy".to_string(), full_name: "Cindy Johnson".to_string() },
            FinalUser { id: 3, user_name: "rich".to_string(), full_name: "Richard Stone".to_string() }
        ])
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .service(
                web::scope("/v1")
                    .service(
                        web::resource("/user/{id}")
                            .route(web::get().to(get_user_name))
                    )
                    .service(
                        web::resource("/user")
                            .route(web::post().to(insert_user))
                    )
            )
    })
    .bind(("127.0.0.1", 8001))?
    .run()
    .await
}

async fn get_user_name(app_data: web::Data<AppState>, params: Path<EntityId>) -> String {
    app_data.users
        .read().unwrap()
        .iter()
        .find(|user| user.id == params.id).unwrap()
        .clone()
        .user_name
}

async fn insert_user(app_data: web::Data<AppState>, new_user: Json<NewUser>) -> String {
    let mut users = app_data.users.write().unwrap();
    let max_user_id = users.iter().max_by_key(|usr| { usr.id }).unwrap().id;
    users.push(FinalUser { id: max_user_id + 1, user_name: new_user.user_name.clone(), full_name: new_user.full_name.clone() });

    users.last().unwrap().id.to_string()
}
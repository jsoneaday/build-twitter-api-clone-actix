use actix_web::{HttpServer, App, web::{self, Redirect}, web::Path, Responder, web::Data, http::{StatusCode, header::ContentType}, ResponseError, Either, body::BoxBody, HttpResponse};
use serde::Serialize;
use std::sync::RwLock;
use derive_more::{ Display, Error };

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_data = Data::new(
        AppState {
            users: RwLock::new(vec![
                User { id: 1, user_name: "tim".to_string(), full_name: "Tim Johnson".to_string() },
                User { id: 2, user_name: "jon".to_string(), full_name: "John Bond".to_string() },
                User { id: 3, user_name: "chris".to_string(), full_name: "Christina Simon".to_string() },
            ])
        }
    );

    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .service(
                web::resource("/user/{user_name}")
                    .route(web::get().to(get_user))
            )  
            .service(
                web::resource("/na")
                    .route(web::get().to(failure_msg))
            )  
            
    })
    .bind(("127.0.0.1", 8001))?
    .run()
    .await
}

#[derive(Clone, Serialize)]
struct User {
    pub id: i64,
    pub user_name: String,
    pub full_name: String
}

impl Responder for User {
    type Body = BoxBody;

    fn respond_to(self, _: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        let user_result = serde_json::to_string(&self);

        match user_result {
            Ok(usr) => {
                HttpResponse::Ok()
                    .content_type(ContentType::json())
                    .body(usr)
            },
            Err(_) => {
                HttpResponse::InternalServerError()
                    .content_type(ContentType::json())
                    .body("Failed to serialize User")
            }
        }
    }
}

struct AppState {
    pub users: RwLock<Vec<User>>
}

#[allow(unused)]
#[derive(Debug, Display, Error)]
enum MyError {
    #[display(fmt = "Internal error")]
    Internal,
    #[display(fmt = "Unknown error")]
    Unknown
}

impl ResponseError for MyError {
    fn status_code(&self) -> StatusCode {
        match *self {
            MyError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            MyError::Unknown => StatusCode::NOT_FOUND
        }
    }
}

async fn get_user(app_data: Data<AppState>, path: Path<String>) -> Either<Result<impl Responder, MyError>, Result<User, MyError>> {   
    println!("start get_user");
    let user_name = path.into_inner();

    let users = app_data.users.read().unwrap();
    let data = users
        .iter()
        .find(|usr| usr.user_name == user_name);

    match data {
        Some(usr) if usr.id != 3 => Either::Left(Ok(Redirect::new("/", "../na"))),
        Some(usr) => Either::Right(Ok(usr.to_owned())),
        None => Either::Right(Err(MyError::Internal))
    }
}

async fn failure_msg() -> &'static str {
    println!("start failure_msg");
    "Something went wrong"
}
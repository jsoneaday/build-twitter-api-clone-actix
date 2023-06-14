use std::sync::RwLock;
use actix_web::{HttpServer, App, Responder, body::BoxBody, HttpResponse, http::header::ContentType, web::{Path, Redirect}, ResponseError, Either};
use serde::Serialize;
use derive_more::{Display, Error};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_data = actix_web::web::Data::new(
        AppState {
            users: RwLock::new(vec![
                User { id: 1, user_name: "jim".to_string(), full_name: "Jim Chan".to_string() },
                User { id: 2, user_name: "tim".to_string(), full_name: "Tim Tom".to_string() },
                User { id: 3, user_name: "lynn".to_string(), full_name: "Lynn Swim".to_string() }
            ])
        }
    );

    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .service(
                actix_web::web::resource("/user/{user_name}")
                    .route(actix_web::web::get().to(get_user))                    
            )
            .service(
                actix_web::web::resource("/na")
                    .route(actix_web::web::get().to(failure_msg))  
            )
    })
    .bind(("127.0.0.1", 8001))?
    .run()
    .await
}

struct AppState {
    users: RwLock<Vec<User>>
}

#[derive(Clone, Serialize)]
pub struct User {
    pub id: i64,
    pub user_name: String,
    pub full_name: String
}

impl Responder for User {
    type Body = BoxBody;

    fn respond_to(self, _: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
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

#[allow(unused)]
#[derive(Debug, Display, Error)]
enum MyError {
    #[display(fmt = "Internal error")]
    Internal,
    #[display(fmt = "Unknown error")]
    Unknown
}

impl ResponseError for MyError {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::build(self.status_code())
            .content_type(ContentType::json())
            .body(self.to_string())
    }

    fn status_code(&self) -> actix_web::http::StatusCode {
        match *self {
            MyError::Internal => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            MyError::Unknown => actix_web::http::StatusCode::NOT_FOUND
        }
    }
}

async fn get_user(app_data: actix_web::web::Data<AppState>, path: Path<String>) 
    -> Either<Result<impl Responder, MyError>, Result<User, MyError>>  {
    let user_name = path.into_inner();

    let users = app_data.users.read().unwrap();
    let user_result = users
        .iter()
        .find(|usr| usr.user_name == user_name);

    match user_result {
        Some(user) if user.id != 3 => Either::Left(Ok(Redirect::new("/", "../na"))),
        Some(user) => Either::Right(Ok(user.clone())),
        None => Either::Right(Err(MyError::Internal))
    }    
}

async fn failure_msg() -> &'static str {
    "Unknown error has occurred"
}
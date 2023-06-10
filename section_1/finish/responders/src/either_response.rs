use actix_web::{Responder, body::BoxBody, HttpResponse, http::header::ContentType, Either, web::Path};
use serde::Serialize;



#[derive(Serialize)]
pub struct LeftType {
    pub left: String
}
impl Responder for LeftType {
    type Body = BoxBody;

    fn respond_to(self, _: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        let result = serde_json::to_string(&self);

        match result {
            Ok(left) => HttpResponse::Ok().content_type(ContentType::json()).body(left),
            Err(_) => HttpResponse::InternalServerError().content_type(ContentType::plaintext()).body("Failed!")
        }
    }
}

#[derive(Serialize)]
pub struct RightType {
    pub right: String
}
impl Responder for RightType {
    type Body = BoxBody;

    fn respond_to(self, _: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        let result = serde_json::to_string(&self);

        match result {
            Ok(right) => HttpResponse::Ok().content_type(ContentType::json()).body(right),
            Err(_) => HttpResponse::InternalServerError().content_type(ContentType::plaintext()).body("Failed!")
        }
    }
}

pub async fn get_profile_name(side: Path<String>) -> Either<LeftType, RightType> {
    if side.contains("left") {
        Either::Left(LeftType { left: "L".to_string() })
    } else {
        Either::Right(RightType { right: "R".to_string() })
    }
}
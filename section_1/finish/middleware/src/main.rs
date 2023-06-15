use std::{future::{ ready, Ready }, sync::Arc};
use actix_web::{
    HttpServer, 
    App, 
    Error,
    web::{post, Bytes, Json}, 
    dev::{Transform, ServiceRequest, Service, ServiceResponse, forward_ready, Payload}, body::{BoxBody, MessageBody}, HttpResponse, HttpRequest, FromRequest, http::header::ContentType
};
use actix_http::{h1, HttpMessage};
use futures_util::future::LocalBoxFuture;
use serde::{Deserialize, Serialize};

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

async fn do_it(data: Json<RequestBodyJson>) -> String {
    println!("start do_it");
    data.msg.clone()
}

struct AppenderMiddlewareBuilder;

impl<S, B> Transform<S, ServiceRequest> for AppenderMiddlewareBuilder 
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AppenderMiddlewareExecutor<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AppenderMiddlewareExecutor { service: Arc::new(service) }))
    }
}

struct AppenderMiddlewareExecutor<S> {
    service: Arc<S>
}

impl<S, B> Service<ServiceRequest> for AppenderMiddlewareExecutor<S> 
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {  
        let fut = self.service.clone();

        Box::pin(async move {
            let body_original = req.extract::<Bytes>().await.unwrap();
            if req.content_type() == ContentType::json().0 {                               
                let body_str = String::from_utf8(body_original.to_vec());
                match body_str {
                    Ok(str) => {
                        let req_body_json: Result<RequestBodyJson, serde_json::Error> = serde_json::from_str(&str);
                        match req_body_json {
                            Ok(mut rbj) => {
                                rbj.msg = format!("{}. how are you?", rbj.msg);
                                let new_rbj_result = serde_json::to_string(&rbj);
                                let new_rbj_str = new_rbj_result.unwrap().clone();
                                println!("new_rbj_str {}", new_rbj_str.clone());
                                let body_final = Bytes::from(new_rbj_str.clone());
                                req.set_payload(bytes_to_payload(body_final));
                            },
                            Err(err) => println!("Failed to deserialize JSON: {}", err)
                        };
                    },
                    Err(err) => println!("Failed to convert bytes to string: {}", err)
                };
            }            
            
            let updated_req_str = String::from_utf8(body_original.to_vec()).unwrap();
            println!("changed req payload {}", updated_req_str);

            let res = fut.call(req).await?;            
            Ok(res)
        })
    }
}

#[derive(Deserialize, Serialize, Clone)]
struct RequestBodyJson {
    pub msg: String
}

fn bytes_to_payload(buf: Bytes) -> Payload {
    let (_, mut pl) = h1::Payload::create(true);
    pl.unread_data(buf);
    Payload::from(pl)
}
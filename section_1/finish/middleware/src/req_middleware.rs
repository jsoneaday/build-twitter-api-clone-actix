use actix_http::{h1, HttpMessage};
use actix_web::{
    Error,
    web::{Bytes, Json}, 
    dev::{Transform, ServiceRequest, Service, ServiceResponse, forward_ready, Payload}, http::header::ContentType
};
use futures_util::future::LocalBoxFuture;
use serde::{Deserialize, Serialize};
use std::{future::{ ready, Ready }, sync::Arc};

pub async fn do_it(data: Json<RequestBodyJson>) -> String {
    data.msg.clone()
}

pub struct AppenderMiddlewareBuilder;

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
        ready(Ok(AppenderMiddlewareExecutor { next_service: Arc::new(service) }))
    }
}

pub struct AppenderMiddlewareExecutor<S> {
    next_service: Arc<S>
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

    forward_ready!(next_service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {  
        let fut = self.next_service.clone();

        Box::pin(async move {
            // warning: extract performs a take and moves data
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
                                let body_final = Bytes::from(new_rbj_str.clone());
                                req.set_payload(bytes_to_payload(body_final));
                            },
                            Err(_) => println!("Not of type RequestBodyJson, continuing")
                        };
                    },
                    Err(_) => println!("Payload not string, continuing")
                };
            }            
            
            let res = fut.call(req).await?;            
            Ok(res)
        })
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct RequestBodyJson {
    pub msg: String
}

fn bytes_to_payload(buf: Bytes) -> Payload {
    let (_, mut pl) = h1::Payload::create(true);
    pl.unread_data(buf);
    Payload::from(pl)
}
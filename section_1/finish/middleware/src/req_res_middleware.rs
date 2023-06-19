use actix_http::{h1, HttpMessage, body::{MessageBody, BoxBody, EitherBody}};
use actix_web::{
    Error,
    web::{Bytes, Json}, 
    dev::{Transform, ServiceRequest, Service, ServiceResponse, forward_ready, Payload}, 
    http::header::ContentType, Responder, HttpResponse
};
use futures_util::future::LocalBoxFuture;
use log::{info, error};
use serde::{Deserialize, Serialize};
use std::{future::{ ready, Ready }, rc::Rc};
use derive_more::{Display, Error};

pub async fn do_it(data: Json<RequestMessage>) -> impl Responder {
    Json(ResponseMessage {
        msg: data.msg.clone()
    })
}

pub struct ReqAppenderMiddlewareBuilder;

impl<S, B> Transform<S, ServiceRequest> for ReqAppenderMiddlewareBuilder 
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static + MessageBody
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type InitError = ();
    type Transform = ReqAppenderMiddlewareExecutor<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ReqAppenderMiddlewareExecutor { next_service: Rc::new(service) }))
    }
}

pub struct ReqAppenderMiddlewareExecutor<S> {
    next_service: Rc<S>
}

impl<S, B> Service<ServiceRequest> for ReqAppenderMiddlewareExecutor<S> 
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static + MessageBody
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(next_service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        info!("enter call");
        let fut = self.next_service.clone();

        Box::pin(async move {
            let body_original = req.extract::<Bytes>().await.unwrap();

            if req.content_type() == ContentType::json().0 {                               
                let body_str = String::from_utf8(body_original.to_vec());
                match body_str {
                    Ok(str) => {
                        let req_body_json: Result<RequestMessage, serde_json::Error> = serde_json::from_str(&str);
                        match req_body_json {
                            Ok(mut rbj) => {
                                info!("update request");
                                rbj.msg = format!("{}. I modified the request", rbj.msg);
                                let new_rbj_result = serde_json::to_string(&rbj);
                                let new_rbj_str = new_rbj_result.unwrap();
                                let body_final = Bytes::from(new_rbj_str);
                                req.set_payload(bytes_to_payload(body_final));
                            },
                            Err(_) => println!("Not of type RequestMessage, continuing")
                        };
                    },
                    Err(_) => println!("Payload not string, continuing")
                };
            }            
            
            let res = fut.call(req).await?;
            if res.headers().clone().contains_key("content-type") {
                let status = res.status();
                let headers = res.headers().clone();
                let request = res.request().clone();
                let body = res.into_body();
                
                let body_bytes_result = body.try_into_bytes();             
                match body_bytes_result {
                    Ok(body_bytes) => {
                        let body_str = String::from_utf8(body_bytes.to_vec());
                        match body_str {
                            Ok(str) => {
                                let body_obj_result: Result<ResponseMessage, serde_json::Error> = serde_json::from_str(&str);
                                match body_obj_result {
                                    Ok(mut body_obj) => {
                                        info!("update response");
                                        body_obj.msg = format!("{}. I modified the response.", body_obj.msg);
                                        
                                        let mut resp = HttpResponse::build(status)
                                            .json(body_obj);
                                        headers.iter().for_each(|(key, value)| {
                                            resp.headers_mut().insert(key.clone(), value.clone());
                                        });                                        
                                        let new_res = ServiceResponse::new(request, resp);
                                        Ok(new_res.map_into_right_body())
                                    },
                                    Err(_) => {
                                        error!("Not of type ResponseMessage, continuing");
                                        Err(MiddleWareError.into()) 
                                    }
                                }
                            },
                            Err(_) => {
                                error!("Payload not string, continuing");
                                Err(MiddleWareError.into()) 
                            }
                        }                          
                    },
                    Err(_) => {
                        error!("Payload not bytes, continuing");
                        Err(MiddleWareError.into()) 
                    }
                }
            } else {
                Ok(res.map_into_left_body())
            }
        })
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct RequestMessage {
    pub msg: String
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ResponseMessage {
    pub msg: String
}

#[derive(Debug, Error, Display)]
struct MiddleWareError;
impl actix_web::ResponseError for MiddleWareError{}

fn bytes_to_payload(buf: Bytes) -> Payload {
    let (_, mut pl) = h1::Payload::create(true);
    pl.unread_data(buf);
    Payload::from(pl)
}

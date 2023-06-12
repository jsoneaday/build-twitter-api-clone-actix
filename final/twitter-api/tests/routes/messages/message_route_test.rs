use actix_http::header::HeaderValue;
use actix_web::http::header;
use fake::Fake;
use fake::faker::internet::en::Username;
use twitter_clone_api::common_tests::actix_fixture::{
    get_profile_create_multipart,
    get_profile_avatar, get_fake_message_body,
};
use twitter_clone_api::routes::output_id::OutputId;
use twitter_clone_api::{
    common_tests::actix_fixture::get_app,
    routes::messages::model::MessageResponder,
};
use twitter_clone_api::routes::messages::model::{ MessagePostJson, MessageGroupTypes, MessageResponders, MessageByFollowingQuery };
use actix_web::{ test, web::Json };
use chrono::Utc;

#[tokio::test]
pub async fn test_route_create_and_get_message() {
    let app = get_app().await;
    let avatar = get_profile_avatar();
    let boundary = Username().fake::<String>(); // use random name as boundary
    let payload = get_profile_create_multipart(&avatar, &boundary, true);

    let header_value_string = format!("multipart/form-data; boundary={}", boundary);
    let header_value = HeaderValue::from_str(&header_value_string);
    let update_avatar_req = test::TestRequest
        ::post()
        .append_header((header::CONTENT_TYPE, header_value.unwrap()))
        .uri("/v1/profile")
        .set_payload(payload)
        .to_request();
    let profile_id_result = test::call_and_read_body_json::<_, _, OutputId>(&app, update_avatar_req).await;
    
    let msg_body: String = get_fake_message_body(None);
    let create_msg_req = test::TestRequest
        ::post()
        .uri("/v1/msg")
        .set_json(
            Json(MessagePostJson {
                user_id: profile_id_result.id,
                body: msg_body.clone(),
                group_type: MessageGroupTypes::Public,
                broadcasting_msg_id: None,
            })
        )
        .to_request();
    let msg_id_result = test::call_and_read_body_json::<_, _, OutputId>(&app, create_msg_req).await;

    let get_msg_req = test::TestRequest::get().uri(&format!("/v1/msg/{}", msg_id_result.id)).to_request();
    let get_msg_body = test::call_and_read_body_json::<_, _, Option<MessageResponder>>(
        &app,
        get_msg_req
    ).await;

    assert!(get_msg_body.unwrap().body.unwrap().eq(&msg_body));
}

#[tokio::test]
pub async fn test_route_create_and_get_messages() {
    let app = get_app().await;
    let avatar = get_profile_avatar();
    let boundary = Username().fake::<String>();
    let payload = get_profile_create_multipart(&avatar, &boundary, true);

    let header_value_string = format!("multipart/form-data; boundary={}", boundary);
    let header_value = HeaderValue::from_str(&header_value_string);
    let update_avatar_req = test::TestRequest
        ::post()
        .append_header((header::CONTENT_TYPE, header_value.unwrap()))
        .uri("/v1/profile")
        .set_payload(payload)
        .to_request();
    let profile_id_result = test::call_and_read_body_json::<_, _, OutputId>(&app, update_avatar_req).await;
    
    let msg_body: String = get_fake_message_body(None);
    let create_msg_req = test::TestRequest
        ::post()
        .uri("/v1/msg")
        .set_json(
            Json(MessagePostJson {
                user_id: profile_id_result.id,
                body: msg_body.clone(),
                group_type: MessageGroupTypes::Public,
                broadcasting_msg_id: None,
            })
        )
        .to_request();
    _ = test::call_and_read_body_json::<_, _, OutputId>(&app, create_msg_req).await;

    println!("build get_messages request: {} {} {}", profile_id_result.id, Utc::now().to_rfc3339(), 10);
    let get_messages_req = test::TestRequest
        ::post()
        .uri("/v1/msgs")
        .set_json(
            Json(
                MessageByFollowingQuery {
                    follower_id: profile_id_result.id,
                    last_updated_at: Utc::now(),
                    page_size: Some(10)
                }
            )
        )        
        .to_request();
    let get_messages_result = test::call_and_read_body_json::<_, _, MessageResponders>(
        &app,
        get_messages_req
    ).await;
    println!("messages {:?}", get_messages_result);
    //assert!(get_messages_result.unwrap().body.unwrap().eq(&msg_body));
}
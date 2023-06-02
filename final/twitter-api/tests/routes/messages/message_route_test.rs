use actix_http::header::HeaderValue;
use actix_web::http::header;
use fake::Fake;
use fake::faker::internet::en::Username;
use twitter_clone_api::common_tests::actix_fixture::{
    get_profile_create_multipart,
    get_profile_avatar,
};
use twitter_clone_api::{
    common_tests::actix_fixture::get_app,
    routes::messages::model::MessageResponder,
};
use twitter_clone_api::routes::messages::model::{ MessagePostJson, GroupTypes };
use actix_web::{ test, web::Json };

#[tokio::test]
pub async fn test_route_create_and_get_message() {
    let app = get_app().await;
    let avatar = get_profile_avatar();
    let boundary = Username().fake::<String>(); // use random name as boundary
    let payload = get_profile_create_multipart(&avatar, &boundary, true);

    let header_value_string = format!("multipart/form-data; boundary={}", boundary);
    let header_value = HeaderValue::from_str(&header_value_string);
    println!("start create profile");
    let update_avatar_req = test::TestRequest
        ::post()
        .append_header((header::CONTENT_TYPE, header_value.unwrap()))
        .uri("/v1/profile")
        .set_payload(payload)
        .to_request();
    let profile_id = test::call_and_read_body_json::<_, _, i64>(&app, update_avatar_req).await;
    println!("end create profile");

    const MSG_BODY_STR: &str = "Testing 123";
    println!("start create message");
    let create_msg_req = test::TestRequest
        ::post()
        .uri("/v1/msg")
        .set_json(
            Json(MessagePostJson {
                user_id: profile_id,
                body: MSG_BODY_STR.clone().to_string(),
                group_type: GroupTypes::Public,
                broadcasting_msg_id: None,
            })
        )
        .to_request();
    let msg_id = test::call_and_read_body_json::<_, _, i64>(&app, create_msg_req).await;
    println!("end create message");

    println!("start get message");
    let get_msg_req = test::TestRequest::get().uri(&format!("/v1/msg?id={}", msg_id)).to_request();
    let get_msg_body = test::call_and_read_body_json::<_, _, Option<MessageResponder>>(
        &app,
        get_msg_req
    ).await;
    println!("end get message");

    assert!(get_msg_body.unwrap().body.unwrap().eq(MSG_BODY_STR));
}

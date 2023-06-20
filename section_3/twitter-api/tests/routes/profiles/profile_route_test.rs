use actix_http::header::HeaderValue;
use fake::{ faker::{ internet::en::Username }, Fake };
use twitter_clone_api::{
    routes::{profiles::model::{ ProfileResponder }, output_id::OutputId},
    common_tests::actix_fixture::{ get_profile_create_multipart, get_profile_avatar },
};
use actix_web::{ test, http::header };
use twitter_clone_api::common_tests::actix_fixture::get_app;

#[tokio::test]
async fn test_route_create_profile_with_avatar() {
    let app = get_app().await;
    let avatar = get_profile_avatar();
    let boundary = Username().fake::<String>(); // use random name as boundary
    let payload = get_profile_create_multipart(&avatar, &boundary, true);    
    
    let header_value_string = format!("multipart/form-data; boundary={}", boundary);
    let header_value = HeaderValue::from_str(&header_value_string);
    let update_avatar_req = test::TestRequest // essentially the same thing as Reqwest
        ::post()
        .append_header((header::CONTENT_TYPE, header_value.unwrap()))
        .uri("/v1/profile")
        .set_payload(payload)
        .to_request();
    let user_id_result = test::call_and_read_body_json::<_, _, OutputId>(&app, update_avatar_req).await;

    let get_profile_req = test::TestRequest
        ::get()
        .uri(&format!("/v1/profile/{}", user_id_result.id))
        .to_request();
    let get_profile_result = test
        ::call_and_read_body_json::<_, _, Option<ProfileResponder>>(&app, get_profile_req).await
        .unwrap();

    assert!(get_profile_result.id == user_id_result.id);
    assert!(get_profile_result.avatar.unwrap() == avatar);
}

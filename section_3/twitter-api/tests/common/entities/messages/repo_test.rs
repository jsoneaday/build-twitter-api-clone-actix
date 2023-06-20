use twitter_clone_api::{
    common_tests::actix_fixture::{ get_app_state, PUBLIC_GROUP_TYPE },
    common::entities::{
        profiles::{ model::ProfileCreate, repo::{ InsertProfileFn } },
        messages::repo::{ InsertMessageFn, QueryMessageFn }, base::DbRepo,
    },
};

#[tokio::test]
async fn test_insert_message() {
    let app_data = get_app_state(DbRepo::init().await).await;
    let db_repo = app_data.db_repo;

    const BODY: &str = "Test chatter post";
    let profile_id = db_repo
        .insert_profile(ProfileCreate {
            user_name: "tester".to_string(),
            full_name: "Dave Wave".to_string(),
            description: "a description".to_string(),
            region: Some("usa".to_string()),
            main_url: Some("http://whatever.com".to_string()),
            avatar: Some(vec![]),
        }).await
        .unwrap();

    let message_id = db_repo
        .insert_message(profile_id, BODY, PUBLIC_GROUP_TYPE, None).await
        .unwrap();

    assert!(message_id > 0);
}

#[tokio::test]
async fn test_query_message() {
    let app_data = get_app_state(DbRepo::init().await).await;
    let db_repo = app_data.db_repo;

    const BODY: &str = "Test chatter post";
    let profile_id = db_repo
        .insert_profile(ProfileCreate {
            user_name: "tester".to_string(),
            full_name: "Dave Wave".to_string(),
            description: "a description".to_string(),
            region: Some("usa".to_string()),
            main_url: Some("http://whatever.com".to_string()),
            avatar: Some(vec![]),
        }).await
        .unwrap();

    let message_id = db_repo
        .insert_message(profile_id, BODY, PUBLIC_GROUP_TYPE, None).await
        .unwrap();
    assert!(message_id > 0);

    let message = db_repo.query_message(message_id).await.unwrap();
    assert!(message.is_some() == true);
}

use crate::helpers::{assert_is_redirect_to, spawn_app};

#[tokio::test]
async fn an_error_flash_message_is_set_on_failure() {
    // arrange
    let app = spawn_app().await;
    let login_body = serde_json::json!({
       "username": "random-username",
        "password": "random-password"
    });

    // act
    let response = app.post_login(&login_body).await;
    let flash_cookie = response.cookies().find(|c| c.name() == "_flash").unwrap();
    let html_page = app.get_login_html().await;

    // assert
    assert_is_redirect_to(&response, "/login");
    assert_eq!(flash_cookie.value(), "Authentication failed");
    assert!(html_page.contains(r#"<p><i>Authentication failed</i></p>"#));
}
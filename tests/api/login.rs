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
    let html_page = app.get_login_html().await;
    let html_page_without_failure = app.get_login_html().await;

    // assert
    assert_is_redirect_to(&response, "/login");
    assert!(html_page.contains(r#"<p><i>Authentication failed</i></p>"#));
    assert!(!html_page_without_failure.contains(r#"<p><i>Authentication failed</i></p>"#));
}

#[tokio::test]
async fn redirect_to_admin_dashboard_after_login_success() {
    // arrange
    let app = spawn_app().await;
    let login_body = serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password
    });

    // act
    let response = app.post_login(&login_body).await;
    let html_page = app.get_admin_dashboard_html().await;

    // assert
    assert_is_redirect_to(&response, "/admin/dashboard");
    assert!(html_page.contains(&format!("Welcome {}", app.test_user.username)));
}
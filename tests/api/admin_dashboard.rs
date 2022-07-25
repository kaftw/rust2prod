use crate::helpers::{assert_is_redirect_to, spawn_app};

#[tokio::test]
async fn you_must_be_logged_in_to_access_the_admin_dashboard() {
    // arrange
    let app = spawn_app().await;

    // act
    let response = app.get_admin_dashboard().await;

    // assert
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn logout_clears_session_state() {
    // arrange
    let app = spawn_app().await;
    let login_body = serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password
    });
    app.post_login(&login_body).await;

    // act
    let response = app.post_logout().await;
    let html_page = app.get_login_html().await;
    let admin_response = app.get_admin_dashboard().await;

    // assert
    assert_is_redirect_to(&response, "/login");
    assert!(html_page.contains(r#"<p><i>You have successfully logged out.</i></p>"#));
    assert_is_redirect_to(&admin_response, "/login");
}
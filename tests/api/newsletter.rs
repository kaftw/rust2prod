use crate::helpers::{TestApp, spawn_app, ConfirmationLinks, assert_is_redirect_to};
use std::time::Duration;
use fake::faker::internet::en::SafeEmail;
use fake::faker::name::en::Name;
use fake::Fake;
use wiremock::matchers::{any, method, path};
use wiremock::{Mock, MockBuilder, ResponseTemplate};

#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    // arrange
    let app = spawn_app().await;
    app.post_login_with_test_user().await;
    create_unconfirmed_subscriber(&app).await;
    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        // We assert that no request is fired at Postmark
        .expect(0)
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
       "title": "Newsletter title",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
        "idempotency_key": uuid::Uuid::new_v4().to_string()
    });

    // act
    let response = app.post_newsletter(&newsletter_request_body).await;
    let html_page = app.get_newsletters_html().await;
    app.dispatch_all_pending_emails().await;

    // assert
    assert_eq!(response.status().as_u16(), 303);
    assert!(html_page.contains(
        "<p><i>The newsletter issue has been accepted - \
        emails will go out shortly.</i></p>"
    ))
    // Mock verifies on Drop that we haven't sent the newsletter email
}

#[tokio::test]
async fn newsletters_are_delivered_to_confirmed_subscribers() {
    // arrange
    let app = spawn_app().await;
    app.post_login_with_test_user().await;
    let confirmation_links = create_unconfirmed_subscriber(&app).await;
    let confirmation_response = reqwest::Client::new()
        .get(confirmation_links.html)
        .send()
        .await
        .unwrap();
    assert_eq!(confirmation_response.status().as_u16(), 200);

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
        "idempotency_key": uuid::Uuid::new_v4().to_string()
    });

    // act
    let response = app.post_newsletter(&newsletter_request_body).await;
    let html_page: String = app.get_newsletters_html().await;
    app.dispatch_all_pending_emails().await;

    // assert
    assert_eq!(response.status().as_u16(), 303);
    assert!(html_page.contains("The newsletter issue has been accepted - \
                                emails will go out shortly."));
    // Mock verifies on Drop that we have sent the newsletter email
}

#[tokio::test]
async fn newsletters_returns_400_for_invalid_data() {
    // arrange
    let app = spawn_app().await;
    app.post_login_with_test_user().await;
    let test_cases = vec![
        (
            serde_json::json!({
                "text_content": "Newsletter body as plain text",
                "html_content": "<p>Newsletter body as HTML</p>"              
            }),
            "missing title"
        ),
        (
            serde_json::json!( { "title": "Newsletter!" } ),
            "missing content"
        )
    ];

    // act
    for (invalid_body, error_message) in test_cases {
        let response = app.post_newsletter(&invalid_body).await;

        // assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

/// Use of the public API of the application under test to create
/// an unconfirmed subscriber.
async fn create_unconfirmed_subscriber(app: &TestApp) -> ConfirmationLinks {
    let name: String = Name().fake();
    let email: String = SafeEmail().fake();
    
    let body = serde_urlencoded::to_string(&serde_json::json!({
        "name": name,
        "email": email
    })).unwrap();
    
    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;

    app.post_subscriptions(body.into())
        .await
        .error_for_status()
        .unwrap();

    let email_request = &app.email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();

    app.get_confirmation_links(&email_request)
}

async fn create_confirmed_subscriber(app: &TestApp) {
    let confirmation_links = create_unconfirmed_subscriber(app).await;
    let client = reqwest::ClientBuilder::default().build().expect("Unable to create client.");
    client.get(confirmation_links.html).send().await.expect("Failed to make request.");
}

#[tokio::test]
async fn newsletter_creation_is_idempotent() {
    // arrange
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;
    app.post_login_with_test_user().await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
        "idempotency_key": uuid::Uuid::new_v4().to_string()
    });

    // act
    let response = app.post_newsletter(&newsletter_request_body).await;
    let html_page = app.get_newsletters_html().await;
    let second_response = app.post_newsletter(&newsletter_request_body).await;
    let updated_html_page = app.get_newsletters_html().await;
    app.dispatch_all_pending_emails().await;

    // assert
    assert_is_redirect_to(&response, "/admin/newsletters");
    assert!(html_page.contains("The newsletter issue has been accepted - \
                                emails will go out shortly."));
    assert_is_redirect_to(&second_response, "/admin/newsletters");
    assert!(updated_html_page.contains("The newsletter issue has been accepted - \
                                        emails will go out shortly."));
}

#[tokio::test]
async fn concurrent_form_submission_is_handled_gracefully() {
    // arrange
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;
    app.post_login_with_test_user().await;
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(2)))        
        .expect(1)
        .mount(&app.email_server)
        .await;

    let newsletter_body = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
        "idempotency_key": uuid::Uuid::new_v4().to_string()
    });

    // act
    let response1 = app.post_newsletter(&newsletter_body);
    let response2 = app.post_newsletter(&newsletter_body);
    let (response1, response2) = tokio::join!(response1, response2);
    app.dispatch_all_pending_emails().await;

    // assert
    assert_eq!(response1.status(), response2.status());
    assert_eq!(response1.text().await.unwrap(), response2.text().await.unwrap());
}

fn when_sending_an_email() -> MockBuilder {
    Mock::given(path("/email")).and(method("POST"))
}

#[tokio::test]
async fn transient_errors_do_not_cause_duplicate_deliveries_on_retries() {
    // arrange
    let app = spawn_app().await;
    let newsletter_body = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
        "idempotency_key": uuid::Uuid::new_v4().to_string()
    });
    create_confirmed_subscriber(&app).await;
    create_confirmed_subscriber(&app).await;
    app.post_login_with_test_user().await;

    // Email delivery fails for the second subscriber
    when_sending_an_email()
        .respond_with(ResponseTemplate::new(200))
        .up_to_n_times(1)
        .expect(1)
        .mount(&app.email_server)
        .await;
    when_sending_an_email()
        .respond_with(ResponseTemplate::new(500))
        .up_to_n_times(1)
        .expect(1)
        .mount(&app.email_server)
        .await;

    // act
    let response1 = app.post_newsletter(&newsletter_body).await;
    
    // Email delivery will succeed for both subscribers now
    when_sending_an_email()
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .named("Delivery retry")
        .mount(&app.email_server)
        .await;

    let response2 = app.post_newsletter(&newsletter_body).await;
    app.dispatch_all_pending_emails().await;

    // assert
    assert_eq!(response1.status().as_u16(), 500);
    assert_eq!(response2.status().as_u16(), 303);
}
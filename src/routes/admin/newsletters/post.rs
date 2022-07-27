use std::fmt::Formatter;
use actix_web::{HttpResponse, ResponseError, web};
use actix_web::body::BoxBody;
use sqlx::PgPool;
use crate::routes::error_chain_fmt;
use actix_web::http::StatusCode;
use crate::email_client::EmailClient;
use anyhow::Context;
use crate::domain::SubscriberEmail;
use actix_web::http::header::HeaderValue;
use reqwest::header;
use crate::authentication::UserId;

#[derive(thiserror::Error)]
pub enum PublishError {
    #[error("Authentication failed.")]
    AuthError(#[source]anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from]anyhow::Error)
}
impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}
impl ResponseError for PublishError {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        match self {
            Self::UnexpectedError(_) => HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR),
            Self::AuthError(_) => {
                let mut response = HttpResponse::new(StatusCode::UNAUTHORIZED);
                let header_value = HeaderValue::from_str(r#"Basic realm="publish""#)
                    .unwrap();
                response.headers_mut().insert(header::WWW_AUTHENTICATE, header_value);
                response
            }
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            Self::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::AuthError(_) => StatusCode::UNAUTHORIZED
        }
    }
}

struct ConfirmedSubscriber {
    email: SubscriberEmail
}

#[derive(serde::Deserialize)]
pub struct BodyData {
    title: String,
    html_content: String,
    text_content: String
}

#[tracing::instrument(
    name = "Publish a newsletter issue",
    skip (body, pool, email_client, user_id),
    fields (user_id = tracing::field::Empty)
)]
pub async fn publish_newsletter(
    body: web::Form<BodyData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    user_id: web::ReqData<UserId>
) -> Result<HttpResponse, PublishError> {    
    let user_id = user_id.into_inner();
    tracing::Span::current().record(
        "user_id",
        &tracing::field::display(*user_id)
    );

    let subscribers = get_confirmed_subscribers(&pool).await?;

    for subscriber in subscribers {
        match subscriber {
            Ok(subscriber) => {
                email_client.send_email(
                    &subscriber.email,
                    &body.title,
                    &body.html_content,
                    &body.text_content
                )
                    .await
                    .with_context(|| {
                        format!("Failed to send newsletter issue to {}", subscriber.email)
                    })?;
            },
            Err(error) => {
                tracing::warn!(
                    error.cause_chain = ?error,
                    "Skipping a confirmed subscriber. \
                    Their stored contact details are invalid."
                )
            }
        }
    }

    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(name = "Get confirmed subscribers", skip(pool))]
async fn get_confirmed_subscribers (
    pool: &PgPool
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    let confirmed_subscribers = sqlx::query!(
        r#"
        SELECT email
        FROM subscriptions
        WHERE status = 'confirmed'
        "#
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| match SubscriberEmail::parse(r.email) {
        Ok(email) => Ok(ConfirmedSubscriber { email }),
        Err(error) => Err(anyhow::anyhow!(error))
    })
    .collect();

    Ok(confirmed_subscribers)
}
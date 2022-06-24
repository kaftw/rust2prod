use std::fmt::Formatter;
use actix_web::{HttpResponse, web};
use actix_web::http::StatusCode;
use sqlx::PgPool;
use uuid::Uuid;
use crate::routes::error_chain_fmt;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String
}

#[derive(thiserror::Error)]
pub enum ConfirmError {
    #[error("The subscription token is invalid.")]
    TokenDoesNotExist,
    #[error(transparent)]
    UnexpectedError(#[from]sqlx::Error)
}
impl std::fmt::Debug for ConfirmError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}
impl actix_web::ResponseError for ConfirmError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::TokenDoesNotExist => StatusCode::UNAUTHORIZED,
            Self::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}


#[tracing::instrument(
    name = "Confirm a pending subscriber"
    skip(parameters, pool)
)]
pub async fn confirm(
    parameters: web::Query<Parameters>,
    pool: web::Data<PgPool>
) -> Result<HttpResponse, ConfirmError> {
    let id = get_subscriber_id_from_token(&pool, &parameters.subscription_token).await?;
    confirm_subscriber(&pool, id).await?;
    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(
    name = "Mark subscriber as confirmed",
    skip(pool, subscriber_id)
)]
async fn confirm_subscriber(
    pool: &PgPool,
    subscriber_id: Uuid
) -> Result<(), ConfirmError> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        subscriber_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

#[tracing::instrument(
    name = "Get subscriber_id from token",
    skip(subscription_token, pool)
)]
async fn get_subscriber_id_from_token(
    pool: &PgPool,
    subscription_token: &String
) -> Result<Uuid, ConfirmError> {
    let result = sqlx::query!(
        r#"SELECT subscriber_id FROM subscription_tokens WHERE subscription_token = $1"#,
        subscription_token
    )
    .fetch_optional(pool)
    .await?;
    match result.map(|r| r.subscriber_id) {
        Some(subscriber_id) => Ok(subscriber_id),
        None => Err(ConfirmError::TokenDoesNotExist)
    }
}
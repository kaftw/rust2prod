use actix_web::{HttpResponse, web};
use actix_web::http::header::LOCATION;
use secrecy::Secret;
use crate::routes::FormData;

pub async fn login(_form: web::Form<FormData>) -> HttpResponse {
    HttpResponse::Ok()
        .insert_header((LOCATION, "/"))
        .finish()
}
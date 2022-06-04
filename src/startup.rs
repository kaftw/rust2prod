use std::net::TcpListener;
use actix_web::dev::Server;
use actix_web::{App, HttpServer, web};
use actix_web::web::Data;
use tracing_actix_web::TracingLogger;
use sqlx::PgPool;
use crate::routes::{health_check, subscribe};
use crate::email_client::EmailClient;

pub fn run(listener: TcpListener, db_pool: PgPool, email_client: EmailClient) -> Result<Server, std::io::Error> {
    let db_pool = web::Data::new(db_pool);
    let email_client = Data::new(email_client);
    let server = HttpServer::new(move ||
        {
            App::new()
                .wrap(TracingLogger::default())
                .route("/health_check", web::get().to(health_check))
                .route("/subscriptions", web::post().to(subscribe))
                // register the connection as part of the application state
                .app_data(db_pool.clone())
                .app_data(email_client.clone())
        })
        .listen(listener)?
        .run();

    Ok(server)
}
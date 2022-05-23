use std::net::TcpListener;
use sqlx::PgPool;
use rust2prod::configuration::get_configuration;
use rust2prod::startup::run;
use rust2prod::telemetry::{get_subscriber, init_subscriber};
use secrecy::ExposeSecret;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // panic if we can't read config
    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_pool = PgPool::connect(
        &configuration.database.connection_string().expose_secret())
        .await
        .expect("Failed to connect to Postgres.");
    let listener = TcpListener::bind(format!("127.0.0.1:{}", configuration.application_port)).expect("Failed to bind address");
    run(listener, connection_pool)?.await
}

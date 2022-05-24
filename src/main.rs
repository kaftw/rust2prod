use std::net::TcpListener;
use rust2prod::configuration::get_configuration;
use rust2prod::startup::run;
use rust2prod::telemetry::{get_subscriber, init_subscriber};
use secrecy::ExposeSecret;
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // panic if we can't read config
    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_pool = PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_lazy(&configuration.database.connection_string().expose_secret())
        .expect("Failed to connect to Postgres.");
    let listener = TcpListener::bind(format!("{}:{}", configuration.application.host, configuration.application.port))
        .expect("Failed to bind address");
    run(listener, connection_pool)?.await
}

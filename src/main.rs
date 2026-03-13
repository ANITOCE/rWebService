use secrecy::ExposeSecret;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;

use rWebService::configuration::get_configuration;
use rWebService::startup::run;
use rWebService::telemetry::{get_subscriber, init_subscriber};
use rWebService::email_client::EmailClient;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("rWebService".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // Panic if we can't read configuration
    let configuration = get_configuration().expect("Failed to read configuration.");
    // lazy connection will not attempt to connect until first use
    let connection_pool = PgPoolOptions::new().connect_lazy_with(configuration.database.connect_options());

    // Build an `EmailClient` using `configuration`
    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address.");
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        configuration.email_client.authorization_token,
    );

    // We have removed the hard-coded `8000` - it's now coming from our settings!
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address)?;
    // println!("Server running at http://{}", listener.local_addr()?);
    run(listener, connection_pool, email_client)?.await?;
    return Ok(());
}

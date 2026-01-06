use sqlx::PgPool;
use std::net::TcpListener;
use secrecy::ExposeSecret;

use rWebService::configuration::get_configuration;
use rWebService::startup::run;
use rWebService::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    
    let subscriber = get_subscriber("rWebService".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // Panic if we can't read configuration
    let configuration = get_configuration().expect("Failed to read configuration.");
    // lazy connection will not attempt to connect until first use
    let connection_pool = PgPool::connect_lazy(configuration.database.connection_string().expose_secret())
        // .await
        .expect("Failed to connect to Postgres.");

    // We have removed the hard-coded `8000` - it's now coming from our settings!
    let address = format!("{}:{}", configuration.application.host, configuration.application.port);
    let listener = TcpListener::bind(address)?;
    // println!("Server running at http://{}", listener.local_addr()?);
    return run(listener, connection_pool)?.await;
}

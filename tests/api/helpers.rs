use std::sync::LazyLock;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;

use rWebService::configuration::{self, DatabaseSettings, get_configuration};
use rWebService::startup::{Application, get_connection_pool};
use rWebService::telemetry::{get_subscriber, init_subscriber};

// Ensure that the `tracing` stack is only initialised once using `once_cell`
static TRACING: LazyLock<()> = LazyLock::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    // We cannot assign the output of `get_subscriber` to a variable based on the
    // value TEST_LOG` because the sink is part of the type returned by
    // `get_subscriber`, therefore they are not the same type. We could work around
    // it, but this is the most straight-forward way of moving forward.
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    // only for test database cleanup
    // config: DatabaseSettings,
}

// Launch our application in the background ~somehow~
// if we fail to perform the required setup we can just panic and crash
// all the things.
pub async fn spawn_app() -> TestApp {
    // The first time `initialize` is invoked the code in `TRACING` is executed.
    // All other invocations will instead skip execution.
    LazyLock::force(&TRACING);

    // let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    // let port = listener.local_addr().unwrap().port();
    // let address = format!("http://127.0.0.1:{}", port);

    let configuration = {
        let mut config = get_configuration().expect("Failed to read configuration");
        config.database.database_name = format!("test_{}", Uuid::new_v4().to_string());
        // Use a random OS port
        // config.database.port = 0;
        config
    };

    configure_database(&configuration.database).await;

    // Launch the application as a background task
    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application.");
    let address = format!("http://localhost:{}", application.port());
    let _ = tokio::spawn(application.run_until_stopped());
    
    return TestApp {
        address,
        db_pool: get_connection_pool(&configuration.database),
        // config: configuration.database,
    };
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database
    let mut connection = PgConnection::connect_with(&config.connect_options_in_test())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");
    // Migrate database
    let connection_pool = PgPool::connect_with(config.connect_options_in_test())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    return connection_pool;
}

// delete the test database when the TestApp drops
// impl Drop for TestApp {
//     fn drop(&mut self) {
//         // We cannot use `async` code in a `drop` implementation, therefore we need
//         // to block the current thread until the database is deleted.
//         let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
//         rt.block_on(delect_test_database(&self.db_pool));
//     }
// }

pub async fn delect_test_database(config: &DatabaseSettings) {
    let mut connection = PgConnection::connect_with(&config.connect_options_in_test())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"DROP DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to delete database.");
}
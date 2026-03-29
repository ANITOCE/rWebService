use rWebService::email_client;
use secrecy::Secret;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::sync::LazyLock;
use uuid::Uuid;
use wiremock::MockServer;

use rWebService::configuration::{DatabaseSettings, get_configuration};
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
    pub port: u16,
    pub db_pool: PgPool,

    // only for test database cleanup
    // check the test whether they pass
    // pub is_pass: bool,

    pub email_server: MockServer,
}

/// Confirmation links embedded in the request to the email API.
pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        let client = reqwest::Client::new();
        return client
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.");
    }

    pub fn get_confirmation_links(&self, email_request: &wiremock::Request) -> ConfirmationLinks {
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();
        // Extract the link from one of the request fields.
        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();
            assert_eq!(links.len(), 1);
            let raw_link = links[0].as_str().to_owned();
            let mut confirmation_link = reqwest::Url::parse(&raw_link).unwrap();
            // Let's make sure we don't call random APIs on the web
            assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");
            confirmation_link.set_port(Some(self.port)).unwrap();
            confirmation_link
        };
        let html = get_link(&body["HtmlBody"].as_str().unwrap());
        let plain_text = get_link(&body["TextBody"].as_str().unwrap());
        return ConfirmationLinks { html, plain_text };
    }

    // pub fn get_test_whether_pass<F>(&mut self, test: F) -> bool
    // where
    //     F: FnOnce() + std::panic::UnwindSafe,
    // {
    //     self.is_pass = std::panic::catch_unwind(test).is_ok();
    //     return self.is_pass;
    // }
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

    let email_server = MockServer::start().await;

    let configuration = {
        let mut config = get_configuration().expect("Failed to read configuration");
        config.database.database_name = format!("test_{}", Uuid::new_v4().to_string());
        config.application.port = 0;
        config.email_client.base_url = email_server.uri();
        config
    };

    configure_database(&configuration.database).await;

    // Launch the application as a background task
    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application.");
    let application_port = application.port();
    let address = format!("http://localhost:{}", application_port);
    let _ = tokio::spawn(application.run_until_stopped());

    return TestApp {
        address,
        port: application_port,
        db_pool: get_connection_pool(&configuration.database),
        email_server,
        // is_pass: false,
    };
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database

    let maintenance_settings = DatabaseSettings {
        database_name: "postgres".to_string(),
        username: "postgres".to_string(),
        password: Secret::new("liu8381".to_string()),
        ..config.clone()
    };

    // connect to the Postgres instance for creating the test database
    let mut connection = PgConnection::connect_with(&maintenance_settings.connect_options())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");
    // Migrate database
    // connect the test database for running the migrations
    let connection_pool = PgPool::connect_with(config.connect_options())
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
    let mut connection = PgConnection::connect_with(&config.connect_options())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"DROP DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to delete database.");
}

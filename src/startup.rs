use actix_web::{App, HttpResponse, HttpServer, web};
use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::middleware::Logger;
use std::net::TcpListener;
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

use crate::routes::{health_check, subscribe};

// We need to mark `run` as public.
// It is no longer a binary entrypoint, therefore we can mark it as async
// without having to use any proc-macro incantation.
pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    let db_pool = Data::new(db_pool);
    let server = HttpServer::new(move || {
        App::new()
        // Middlewares are added using the `wrap` method on `App`
        .wrap(TracingLogger::default())
        .route("/health_check", web::get().to(health_check))
        .route("/subscriptions", web::post().to(subscribe))
        // Get a pointer copy and attach it to the application state
        .app_data(db_pool.clone())
    }) 
    .listen(listener)?
    .run();
    return Ok(server);
}

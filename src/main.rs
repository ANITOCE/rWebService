use std::net::TcpListener;

use actix_web::{App, HttpRequest, HttpResponse, HttpServer, Responder, web};
use rWebService::run;

async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", &name)
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    return run(TcpListener::bind("127.0.0.1:8000")?)?.await;
}
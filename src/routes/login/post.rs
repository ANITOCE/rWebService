use actix_web::HttpResponse;
use actix_web::http::header::LOCATION;
use actix_web::web;
use secrecy::Secret;

use crate::routes::FormData;

pub async fn login(_form: web::Form<FormData>) -> HttpResponse {
    return HttpResponse::Ok()
        .insert_header((LOCATION, "/"))
        .finish();
}
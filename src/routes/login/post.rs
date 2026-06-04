use actix_web::HttpResponse;

pub async fn login() -> HttpResponse {
    return HttpResponse::Ok().finish();
}
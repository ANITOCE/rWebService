use actix_web::http::header::LOCATION;
use actix_web::HttpResponse;

// Return an opaque 500 while preserving the error root's cause for logging.
pub fn e500<T>(e: T) -> actix_web::Error
where
    T: std::fmt::Debug + std::fmt::Display + 'static,
{
    return actix_web::error::ErrorInternalServerError(e);
}

pub fn see_other(location: &str) -> HttpResponse {
    return HttpResponse::SeeOther()
        .insert_header((LOCATION, location))
        .finish();
}

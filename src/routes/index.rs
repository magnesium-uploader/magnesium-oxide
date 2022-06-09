use actix_web::{Error, HttpRequest, HttpResponse, Result};

/// Frontend index route
pub async fn index(_request: HttpRequest) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("Hello world!"))
}

use actix_web::{web, http::header::LOCATION, HttpResponse};

pub async fn signup() -> Result<HttpResponse, actix_web::Error> {
    Ok(HttpResponse::SeeOther()
       .insert_header((LOCATION, "/blog"))
       .finish()
    )
}

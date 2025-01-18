use actix_web::{web, HttpResponse};
use crate::{
    ssr::SsrCommon,
    util::e500
};

pub async fn get(ssr: web::Data<SsrCommon>) -> Result<HttpResponse, actix_web::Error> {
    let html = ssr.as_ref().clone()
        .render("blog.html")
        .map_err(e500)?;

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(html))
}

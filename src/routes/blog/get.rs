use crate::{flash_message::Flash, ssr::SsrCommon, util::e500};
use actix_session::Session;
use actix_web::{web, HttpResponse};

pub async fn get(
    ssr: web::Data<SsrCommon>,
    session: Session,
) -> Result<HttpResponse, actix_web::Error> {
    let html = if let Some(flash_message) = session.get_flash() {
        session.clear_flash();
        ssr.as_ref().clone().with_context("flash", &flash_message)
    } else {
        ssr.as_ref().clone()
    }
    .render("blog.html")
    .map_err(e500)?;

    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

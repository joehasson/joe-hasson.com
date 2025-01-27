use crate::{flash_message::Flash, ssr::SsrCommon, util::e500};
use actix_session::Session;
use actix_web::{web, HttpResponse};
use serde::Serialize;
use std::fs;

#[derive(Serialize, Debug)]
struct BlogPost {
    content: String,
    path: String, // e.g. "/blog/first-post"
}
pub async fn get(
    ssr: web::Data<SsrCommon>,
    session: Session,
) -> Result<HttpResponse, actix_web::Error> {
    let mut post_titles: Vec<String> = fs::read_dir("blog")?
        .filter_map(|e| {
            e.ok()
                .map(|dir_entry| dir_entry.file_name())
                .and_then(|os_str| os_str.to_str().map(String::from))
        })
        .collect();

    // Sort reverse chronological (post names prefixed by YYYY-MM-DD)
    post_titles.sort_by(|a, b| b.cmp(a));

    let mut posts = vec![];
    for title in &post_titles {
        if let Ok(content) = fs::read_to_string(format!("blog/{}", title)) {
            let post = BlogPost {
                content,
                path: format!("/blog/{}", title),
            };
            posts.push(post)
        }
    }

    let html = if let Some(flash_message) = session.get_flash() {
        session.clear_flash();
        ssr.as_ref().clone().with_context("flash", &flash_message)
    } else {
        ssr.as_ref().clone()
    }
    .with_context("posts", &posts)
    .render("blog.html")
    .map_err(e500)?;

    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

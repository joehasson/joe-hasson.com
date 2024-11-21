use actix_web::{HttpServer, App, web};
use shared::{ssr::SsrCommon, routes};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let ssr_common = web::Data::new(
        SsrCommon::load().expect("Failed to set up SSR")
    );

    HttpServer::new(move || {
        App::new()
            .app_data(ssr_common.clone())
            .route("/health_check", web::get().to(routes::health_check::health_check))
            .route("/blog", web::get().to(routes::blog::blog))
    })
    .bind(("127.0.0.1", 8001))?
    .run()
    .await
}

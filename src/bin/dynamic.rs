use actix_web::{middleware::Logger, web, App, HttpServer};
use shared::{ssr::SsrCommon, routes};
use env_logger;
use log;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    log::info!("Setting up SSR...");
    let ssr_common = web::Data::new(
        SsrCommon::load().expect("Failed to set up SSR")
    );

    log::info!("Setting up server...");
    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(ssr_common.clone())
            .route("/health_check", web::get().to(routes::health_check::health_check))
            .route("/blog", web::get().to(routes::blog::get))
            .route("/blog/signup", web::post().to(routes::blog::post))
    })
    .bind(("127.0.0.1", 8001))?;

    log::info!("Bound to socket succesfully. Starting server...");
    server.run().await
}

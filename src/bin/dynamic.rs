use actix_web::{middleware::Logger, web, App, HttpServer};
use shared::{ssr::SsrCommon, routes};
use dotenvy::dotenv;
use env_logger;
use log;
use sqlx::{postgres::{PgConnectOptions, PgPoolOptions, PgSslMode}, PgPool};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    dotenv().ok();

    log::info!("Setting up SSR...");
    let ssr_common = web::Data::new(
        SsrCommon::load().expect("Failed to set up SSR")
    );

    log::info!("Establishing database connection...");
    let read_env = |varname| {
        std::env::var(varname)
            .expect(format!("Failed to read env var {}", varname).as_ref())
    };

    let options = PgConnectOptions::new()
        .host(&read_env("DB_HOST"))
        .username(&read_env("DB_USER"))
        .password(&read_env("DB_PASSWORD"))
        .port(read_env("DB_PORT").parse::<u16>().expect("DB_PORT was not a u16"))
        .ssl_mode(if read_env("APP_ENV") == "local" { PgSslMode::Prefer } else { PgSslMode::Require});

    let connection_pool = web::Data::new(
        PgPool::connect_with(options)
        .await
        .expect("Failed to establish database connection"));

    log::info!("Setting up server...");
    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(ssr_common.clone())
            .app_data(connection_pool.clone())
            .route("/health_check", web::get().to(routes::health_check::health_check))
            .route("/blog", web::get().to(routes::blog::get))
            .route("/blog/signup", web::post().to(routes::blog::signup))
    })
    .bind(("127.0.0.1", 8001))?;

    log::info!("Bound to socket succesfully. Starting server...");
    server.run().await
}

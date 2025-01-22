use actix_session::storage::CookieSessionStore;
use actix_session::SessionMiddleware;
use actix_web::{cookie::Key, middleware::Logger, web, App, HttpServer};
use core::time::Duration;
use dotenvy::dotenv;
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::AsyncSmtpTransport;
use lettre::transport::smtp::PoolConfig;
use lettre::Tokio1Executor;
use secrecy::{ExposeSecret, Secret};
use shared::{
    email_delivery_worker::worker, email_delivery_worker::EmailClient, routes, ssr::SsrCommon,
    util::read_env_or_panic,
};
use sqlx::{
    postgres::{PgConnectOptions, PgSslMode},
    PgPool,
};
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    dotenv().ok();

    log::info!("Setting up SSR...");
    let ssr_common = web::Data::new(SsrCommon::load().expect("Failed to set up SSR"));

    log::info!("Establishing database connection...");

    let options = PgConnectOptions::new()
        .host(&read_env_or_panic("DB_HOST"))
        .username(&read_env_or_panic("DB_USER"))
        .password(&read_env_or_panic("DB_PASSWORD"))
        .database(&read_env_or_panic("DB_NAME"))
        .port(
            read_env_or_panic("DB_PORT")
                .parse::<u16>()
                .expect("DB_PORT was not a u16"),
        )
        .ssl_mode(PgSslMode::Prefer);

    let pgpool = PgPool::connect_with(options)
        .await
        .expect("Failed to establish database connection");
    let connection_pool = web::Data::new(pgpool.clone());

    log::info!("Setting up email client...");
    let email_address = read_env_or_panic("BLOG_EMAIL_ADDRESS");
    let email_password = Secret::new(read_env_or_panic("BLOG_EMAIL_PASSWORD"));
    let email_creds =
        Credentials::new(email_address.clone(), email_password.expose_secret().into());

    let email_transport = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay("smtp.gmail.com")
        .expect("Failed to create smtp client")
        .credentials(email_creds)
        .timeout(Some(Duration::from_secs(10)))
        .pool_config(PoolConfig::new().max_size(20))
        .port(587)
        .build();

    let email_client = EmailClient::new(Arc::new(email_transport), &email_address)
        .expect("Failed to setup email client");

    log::info!("Setting up email delivery background worker...");
    let worker_task = tokio::spawn(worker(Arc::new(email_client), Arc::new(pgpool)));

    // Set up secret key for flash messaging middleware
    let hmac_secret = Secret::new(read_env_or_panic("APP_HMAC_SECRET"));
    let secret_key = Key::from(hmac_secret.expose_secret().as_bytes());

    log::info!("Setting up server...");
    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .build(),
            )
            .app_data(ssr_common.clone())
            .app_data(connection_pool.clone())
            .route(
                "/health_check",
                web::get().to(routes::health_check::health_check),
            )
            .route("/blog", web::get().to(routes::blog::get))
            .route(
                "/subscriptions",
                web::post()
                    .to(routes::subscriptions::subscribe::<AsyncSmtpTransport<Tokio1Executor>>),
            )
            .route(
                "/subscriptions/confirm",
                web::get().to(routes::subscriptions::confirm::<AsyncSmtpTransport<Tokio1Executor>>),
            )
            .route(
                "/subscriptions/unsubscribe",
                web::get()
                    .to(routes::subscriptions::unsubscribe::<AsyncSmtpTransport<Tokio1Executor>>),
            )
    })
    .bind(("127.0.0.1", 8001))?
    .run();

    log::info!("Bound to socket succesfully. Starting server and background worker...");
    tokio::select! {
        _ = server => {},
        _ = worker_task => {},
    };

    Ok(())
}

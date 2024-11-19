use actix_web::{HttpServer, HttpResponse, App, web};

async fn health_check() -> HttpResponse {
    return HttpResponse::Ok().finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/health_check", web::get().to(health_check))
    })
    .bind(("127.0.0.1", 8001))?
    .run()
    .await
}

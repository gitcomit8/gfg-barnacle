use actix_web::{web, App, HttpServer};
use actix_web::middleware::Logger;

mod auth_middleware;
mod auth_state;
mod handlers;

use auth_middleware::AuthGuard;
use auth_state::AppState;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 Starting Auth Guard Challenge Server...");
    println!("📍 Server running at http://localhost:8080");
    println!("⚠️  WARNING: This module contains a bug that causes infinite redirects!");
    println!();
    println!("Try accessing:");
    println!("  - http://localhost:8080/login (will cause infinite redirects!)");
    println!("  - http://localhost:8080/dashboard (redirects to /login)");
    println!();

    let app_state = web::Data::new(AppState::new());

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(Logger::default())
            // BUGGY: All routes are wrapped with AuthGuard, including /login!
            .wrap(AuthGuard)
            .route("/", web::get().to(handlers::home))
            .route("/login", web::get().to(handlers::login_page))
            .route("/dashboard", web::get().to(handlers::dashboard))
            .service(
                web::scope("/auth")
                    .route("/login", web::post().to(handlers::do_login))
                    .route("/logout", web::post().to(handlers::do_logout))
                    .route("/status", web::get().to(handlers::auth_status))
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

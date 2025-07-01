use axum::{Router, routing::post};
use std::net::SocketAddr;

mod handlers;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/keypair", post(handlers::generate_keypair))
        .route("/token/create", post(handlers::create_token))
        .route("/token/mint", post(handlers::mint_token))
        .route("/message/sign", post(handlers::sign_message))
        .route("/message/verify", post(handlers::verify_message))
        .route("/send/sol", post(handlers::send_sol))
        .route("/send/token", post(handlers::send_token));

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".into());
    let addr = SocketAddr::from(([0, 0, 0, 0], port.parse().unwrap()));
    println!(
        "Server running on 0.0.0.0:{} (env PORT = {})",
        port,
        std::env::var("PORT").unwrap_or_else(|_| "not set".into())
    );
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

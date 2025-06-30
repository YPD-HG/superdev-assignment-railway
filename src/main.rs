use axum::{
    routing::post,
    Router,
};
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


    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running at http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

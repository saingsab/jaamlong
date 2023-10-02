pub mod handlers;
pub mod models;
pub mod routers;
pub mod utils;

use crate::routers::{
    hsm_routes::sign_tx_routes, network_routes::network_routes,
    token_address_routes::token_address_routes, transaction_routes::transaction_routes,
};
use axum::{
    http::{
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
        HeaderValue, Method,
    },
    Router,
};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::sync::Arc;
use tokio::task;
use tower_http::cors::CorsLayer;

pub struct AppState {
    db: Pool<Postgres>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let database_url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = match PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
    {
        Ok(pool) => {
            println!("✅Connection to the database is successful!");
            pool
        }
        Err(err) => {
            println!("🔥 Failed to connect to the database: {:?}", err);
            std::process::exit(1);
        }
    };
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:8000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);
    let app = Router::new()
        .merge(transaction_routes(Arc::new(AppState { db: pool.clone() })))
        .merge(network_routes(Arc::new(AppState { db: pool.clone() })))
        .merge(token_address_routes(Arc::new(AppState {
            db: pool.clone(),
        })))
        .layer(cors);
    let cors2 = CorsLayer::new()
        .allow_origin("http://localhost:7000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);
    let app2 = Router::new().merge(sign_tx_routes()).layer(cors2);
    println!("🚀 Server started successfully, port 8000");
    println!("🚀 HSM Server started successfully, port 7000");
    let server1 = task::spawn(async move {
        axum::Server::bind(&"0.0.0.0:8000".parse().unwrap())
            .serve(app.into_make_service())
            .await
            .unwrap();
    });
    let server2 = task::spawn(async move {
        axum::Server::bind(&"0.0.0.0:7000".parse().unwrap())
            .serve(app2.into_make_service())
            .await
            .unwrap();
    });
    server1.await.unwrap();
    server2.await.unwrap();

    Ok(())
}

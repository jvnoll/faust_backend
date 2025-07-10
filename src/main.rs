mod config;
mod models;
mod dtos;
mod error;
mod db;
mod utils;
mod middleware;

use axum::http::{header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE}, HeaderValue, Method};
use axum::Router;
use config::Config;
use db::DBClient;
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use tokio_cron_scheduler::{Job, JobScheduler};
use tower_http::cors::CorsLayer;
use tracing_subscriber::filter::LevelFilter;

use crate::db::UserExt;

#[derive(Debug, Clone)]
pub struct AppState {
    pub env: Config,
    pub db_client: DBClient,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .init();

    dotenv().ok();
    let config = Config::init();

    let pool = match PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await {
            Ok(pool) => {
                println!("✅Connection to the database was successful!");
                pool
            }
            Err(err) => {
                println!("🔥 Failed to connect to the database: {:?}", err);
                std::process::exit(1);
            }
        };
    
    let cors = CorsLayer::new()
        // Change when needed :3
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE])
        .allow_credentials(true)
        .allow_methods([Method::GET, Method::POST, Method::PUT]);

    let db_client = DBClient::new(pool);
    let app_state = AppState {
        env: config.clone(),
        db_client: db_client.clone(),
    };

    let sched = JobScheduler::new().await.unwrap();
      
    let job = Job::new_async("0 0 * * * *", {  
      move |_, _| {
        let db_client = db_client.clone();
        Box::pin(async move {
            println!("Running scheduled expired file deletion...");
            if let Err(err) = db_client.delete_expired_files().await {
                eprintln!("Error deleting expired files: {:?}", err);
            } else {
                println!("Expired files successfully deleted!")
            }
        })
      }
    }).unwrap();

    sched.add(job).await.unwrap();

    tokio::spawn(async move {
        sched.start().await.unwrap();
    });

    let app = Router::new().layer(cors.clone());
    println!(
        "{}",
        format!("🚀 Server is running on http://localhost:{}", config.port)
    );

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", &config.port))
    .await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
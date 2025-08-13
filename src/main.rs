// BatikKita Backend main.rs - incrementally adding features
mod batik_server;
mod routes;
mod models;
mod utils;
mod middleware;
mod db;
mod seeder;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 Starting BatikKita Backend Server");
    batik_server::run_batik_server().await
}
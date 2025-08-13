mod api_server;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    api_server::run_api_server().await
}

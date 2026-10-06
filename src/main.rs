use dotenvy::dotenv;
use tracing_subscriber::EnvFilter;


use rcx::{
    config::ServerConfig,
    server::Server,
};






#[tokio::main]
async fn main() {
    dotenv().ok();
    
    tracing_subscriber::fmt()
    .with_env_filter(
        EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "rcx=info,tower_http=info".into())
    )
    .init();

    let server_config = ServerConfig::load();
    
    let app = Server::new(&server_config.jwt_secret);

    app.run(&server_config.addr).await;
}

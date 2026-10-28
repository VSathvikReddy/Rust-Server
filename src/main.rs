use dotenvy::dotenv;
use tracing_subscriber::EnvFilter;


use rcx::{
    config::ServerConfig,
    server::Server,
    auth
};
use std::env;
use std::time::Duration;




#[tokio::main]
async fn main() -> std::io::Result<()>{
    dotenv().ok();
    init_tracing();

    let server_config = ServerConfig::load().unwrap();

    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        Server::new()
            .add_rate_limit(&server_config.jwt_secret)
            .build()
            .run(&server_config.addr).await;

        return Ok(());
    }

    auth::gen_passwords_file(
        &args[1], 
        &args[2], 
        &server_config.jwt_secret, 
        Duration::from_hours(server_config.exp)
    )
}



fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rcx=info,tower_http=info".into())
        )
        .init();
}

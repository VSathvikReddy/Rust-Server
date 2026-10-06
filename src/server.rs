use tokio::net::TcpListener;
use tower_governor::{
    GovernorLayer,
    governor::GovernorConfigBuilder
};
use tower_http::{
    trace::TraceLayer,
    cors::{CorsLayer,Any}
};
use axum::http::{header,Method};
use axum::{Router,routing::{get,post},middleware};

use std::{
    net::SocketAddr,
    time::Duration,
    sync::Arc,
};

use super::auth::{JwtKeys,token_verifying_layer,token_maker,UserIdKeyExtractor};

pub struct Server{
    router: Router,
}



async fn server_func() -> String {
    "Hello from Axum!".to_string()
}

impl Server{
    pub fn new(jwt_secret: &str) ->Self{
        let keys= Arc::new(JwtKeys::load(jwt_secret));



        let governor_conf = GovernorConfigBuilder::default()
            .per_second(2)
            .burst_size(5)
            .key_extractor(UserIdKeyExtractor)
            .finish()
            .unwrap();

        

        let governor_limiter = governor_conf.limiter().clone();
    
        let interval = Duration::from_secs(60); // a separate background task to clean up
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(interval);
                tracing::info!("rate limiting storage size: {}", governor_limiter.len());
                governor_limiter.retain_recent();
            }
        });


        let router = Router::new()
            .route("/request", get(server_func))
            .layer(GovernorLayer::new(governor_conf))
            .layer(middleware::from_fn_with_state(keys.clone(), token_verifying_layer));

        let auth_maker:Router= Router::new()
            .route("/auth", post(token_maker))
            .with_state(keys);


        let cors= CorsLayer::new()
            //.allow_origin("http://localhost:5173".parse::<axum::http::HeaderValue>().unwrap())
            .allow_origin(Any)
            .allow_methods([Method::GET,Method::POST])
            .allow_headers([header::AUTHORIZATION,header::CONTENT_TYPE]);
        Self{ 
            router : router.merge(auth_maker)
                .layer(cors)
                .layer(TraceLayer::new_for_http())
        }
    }



    pub async fn run(self, addr: &SocketAddr ){
        tracing::info!("listening on {}", addr);
        let listener = TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, self.router.into_make_service_with_connect_info::<SocketAddr>())
            .with_graceful_shutdown(shutdown_signal())
            .await
            .unwrap();
    }
}

async fn shutdown_signal(){
    let ctrl_c= async{
        tokio::signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    // SIGTERM is what docker/systemd send; it only exists on unix
    #[cfg(unix)]
    let terminate= async{
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate= std::future::pending::<()>();

    // whichever happens first wins
    tokio::select!{
        _= ctrl_c=>{},
        _= terminate=>{},
    }
    tracing::info!("shutdown signal received, draining connections");
}
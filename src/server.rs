use axum::{
    extract::{Path, Json},
    http::StatusCode,
    middleware,
    routing::{get, post},
    Router,
};
use serde_json::json;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::net::TcpListener;
use tower_governor::{
    governor::GovernorConfigBuilder,
    GovernorLayer,
};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use super::auth::{
    token_maker, token_verifying_layer, JwtKeys, UserIdKeyExtractor,
};

const QUESTIONS: [&str; 5] = [
    "Reverse the following string.\nInput: hello_world\nOutput: dlrow_olleh",
    "Return the sum of all integers in the input.\nInput: 10 20 30 40\nOutput: 100",
    "Find the largest value.\nInput: 4 17 2 31 9\nOutput: 31",
    "Count the number of set bits.\nInput: 29\nOutput: 4",
    "Return the Fibonacci number at index n.\nInput: 10\nOutput: 55",
];

pub struct Server {
    router: Router,
}


async fn load_questions() -> Json<Vec<&'static str>> {
    Json(QUESTIONS.to_vec())
}

async fn ask_question(
    Path(index): Path<usize>,
    Json(input): Json<String>,
) -> Result<
    Json<serde_json::Value>,
    (StatusCode, Json<serde_json::Value>)
> {
    if index >= QUESTIONS.len() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "question_not_found",
                "message": "question index is out of range"
            })),
        ));
    }

    let output = match index {
        0 => input.chars().rev().collect::<String>(),
        1 => input
            .split_whitespace()
            .filter_map(|x| x.parse::<i64>().ok())
            .sum::<i64>()
            .to_string(),
        2 => input
            .split_whitespace()
            .filter_map(|x| x.parse::<i64>().ok())
            .max()
            .map(|x| x.to_string())
            .unwrap_or_else(|| "no numbers".to_string()),
        3 => input
            .trim()
            .parse::<u64>()
            .map(|x| x.count_ones().to_string())
            .unwrap_or_else(|_| "invalid integer".to_string()),
        4 => input
            .trim()
            .parse::<u64>()
            .map(fibonacci)
            .map(|x| x.to_string())
            .unwrap_or_else(|_| "invalid integer".to_string()),
        _ => unreachable!(),
    };

    Ok(Json(json!({
        "question": index,
        "output": output
    })))
}

fn fibonacci(n: u64) -> u64 {
    let mut a = 0;
    let mut b = 1;

    for _ in 0..n {
        (a, b) = (b, a + b);
    }

    a
}

impl Server {
    pub fn new() -> Self {
        let router = Router::new()
            .route("/load", get(load_questions))
            .route("/ask/{index}", post(ask_question));
    
        Self {
            router
        }
    }

    pub fn add_rate_limit(
        self,
        jwt_secret: &str,
        distribute_keys: bool,
    ) -> Self {
        let keys = Arc::new(JwtKeys::load(jwt_secret));

        let governor_conf = GovernorConfigBuilder::default()
            .per_second(2)
            .burst_size(5)
            .key_extractor(UserIdKeyExtractor)
            .finish()
            .unwrap();

        let governor_limiter = governor_conf.limiter().clone();

        std::thread::spawn(move || {
            loop {
                std::thread::sleep(Duration::from_secs(60));

                tracing::info!(
                    "rate limiting storage size: {}",
                    governor_limiter.len()
                );

                governor_limiter.retain_recent();
            }
        });

        let protected_router=self.router
            .layer(GovernorLayer::new(governor_conf))
            .layer(middleware::from_fn_with_state(
                keys.clone(),
                token_verifying_layer,
            ));

        // /auth must remain public because it issues the JWT.
        if distribute_keys {
            let auth_router = Router::new()
                .route("/auth", post(token_maker))
                .with_state(keys);

            return Self {
                router: protected_router.merge(auth_router),
            };
        }

        Self {
            router: protected_router,
        }
    }

    pub fn build(self) -> Self {
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        Self {
            router: self
                .router
                .layer(cors)
                .layer(TraceLayer::new_for_http()),
        }
    }

    pub async fn run(self, addr: &SocketAddr) {
        tracing::info!("listening on {}", addr);

        let listener = TcpListener::bind(addr)
            .await
            .expect("failed to bind server address");

        axum::serve(
            listener,
            self.router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server failed");
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(
            tokio::signal::unix::SignalKind::terminate(),
        )
        .expect("failed to install SIGTERM handler")
        .recv()
        .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("shutdown signal received, draining connections");
}

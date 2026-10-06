use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize,Serialize};
use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use tower_governor::{
    key_extractor::KeyExtractor,
    GovernorError,
};
use std::{sync::Arc,time::{SystemTime,UNIX_EPOCH}};


pub(crate) struct JwtKeys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}


impl JwtKeys{
    pub(crate) fn load(secret: &str) -> Self{
        Self{
            encoding: EncodingKey::from_secret(secret.as_bytes()),
            decoding: DecodingKey::from_secret(secret.as_bytes()),
        }
    }
}

#[derive(Serialize,Deserialize,Clone)]
struct Claims{
    id: u64,
    exp: usize,
}



#[derive(Clone)]
pub(crate) struct UserIdKeyExtractor;

impl KeyExtractor for UserIdKeyExtractor {
    type Key = u64;

    fn extract<B>(&self, req: &Request<B>) -> Result<Self::Key, GovernorError> {
        req.extensions()
            .get::<Claims>()
            .map(|claims| claims.id)
            .ok_or(GovernorError::UnableToExtractKey)
    }
}





pub(crate) async fn token_verifying_layer(
    State(keys): State<Arc<JwtKeys>>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {

    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let validation = Validation::default(); // Validates `exp` automatically
    let token_data = decode::<Claims>(token, &keys.decoding, &validation)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
       

    req.extensions_mut().insert(token_data.claims);

    Ok(next.run(req).await)
}


pub(crate) async fn token_maker(
    State(keys): State<Arc<JwtKeys>>
) -> Result<String,StatusCode>{
    // exp is seconds since the epoch, which is what Validation checks against
    let now= SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .as_secs();

    let claims= Claims{
        id: rand::random::<u64>(),
        exp: (now+ 3600) as usize,
    };

    encode(&Header::default(),&claims,&keys.encoding)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
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


use std::{
    sync::Arc,
    time::{SystemTime,UNIX_EPOCH},
    fs::File,
    io
};
use core::time::Duration;

use csv::{Writer,Reader};


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

    pub(crate) fn create_token(&self, mail_id: String, exp: Duration) -> Result<String,StatusCode>{
        let now= SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        
        let claims= Claims{
            mail_id,
            exp: (now + exp).as_secs() as usize
        };

        encode(&Header::default(),&claims,&self.encoding)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }
}

#[derive(Serialize,Deserialize,Clone)]
struct Claims{
    mail_id: String,
    exp: usize,
}



#[derive(Clone)]
pub(crate) struct UserIdKeyExtractor;

impl KeyExtractor for UserIdKeyExtractor {
    type Key = String;

    fn extract<B>(&self, req: &Request<B>) -> Result<Self::Key, GovernorError> {
        req.extensions()
            .get::<Claims>()
            .map(|claims| claims.mail_id.clone())
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


pub fn gen_passwords_file(input_file_path: &str, output_file_name: &str, jwt_secret: &str, exp: Duration) -> io::Result<()> {
    // Open CSV reader and writer
    let mut rdr = Reader::from_path(input_file_path)?;
    let output_file = File::create(output_file_name)?;
    let mut csv_writer = Writer::from_writer(output_file);

    // Read and clone headers to find the email column index dynamically
    let headers = rdr.headers()?.clone();
    let mail_idx = headers.iter().position(|h| {
        let h_lower = h.to_lowercase();
        h_lower == "mail" || h_lower == "mail_id" || h_lower == "email"
    }).ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Could not find email column (looked for 'mail', 'mail_id', 'email')"))?;

    // Write new headers: original headers + "Password"
    let mut new_headers: Vec<String> = headers.iter().map(|s| s.to_string()).collect();
    new_headers.push("Password".to_string());
    csv_writer.write_record(&new_headers)?;

    let key = JwtKeys::load(jwt_secret);
    
    for result in rdr.records() {
        let record = match result {
            Ok(rec) => rec,
            Err(e) => {
                eprintln!("Error reading CSV record: {}", e);
                continue; // Report and continue on read failure
            }
        };

        // Extract the mail id from the detected column index
        let mail_id = match record.get(mail_idx) {
            Some(m) if !m.trim().is_empty() => m.trim(),
            _ => {
                eprintln!("Skipping row: missing or empty mail field");
                continue;
            }
        };

        // Generate the token/password
        let password = match key.create_token(mail_id.to_string(), exp) {
            Ok(password) => password,
            Err(e) => {
                eprintln!("Unable to create password for {}: {}", mail_id, e);
                continue; // Report and continue on token generation failure
            }
        };

        // Collect existing row fields and append the new password
        let mut new_row: Vec<&str> = record.iter().collect();
        new_row.push(&password);

        if let Err(e) = csv_writer.write_record(&new_row) {
            eprintln!("Error writing record for {}: {}", mail_id, e);
        }
    }

    csv_writer.flush()
}
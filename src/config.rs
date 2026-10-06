use std::{net::SocketAddr,env};

pub struct ServerConfig{
    pub host: String,
    pub port: u16,
    pub addr: SocketAddr,
    pub jwt_secret: String,
}

impl ServerConfig{
    pub fn load()->Self{
        

        let host = env::var("SERVER_HOST")
            .expect("SERVER_HOST environment variable is not set");

        let port: u16 = env::var("SERVER_PORT")
            .expect("SERVER_PORT environment variable is not set")
            .parse()
            .expect("SERVER_PORT environment variable is not a valid u16 number");

        let addr: SocketAddr = format!("{}:{}", host, port)
            .parse()
            .expect("Invalid SERVER_HOST and PORT combination");


            
            

        let jwt_secret = env::var("JWT_SECRET")
            .expect("JWT_SECRET environment variable missing");
      
    
        ServerConfig{
            host,
            port,
            addr,
            jwt_secret,
        }
    }
}
use std::{net::SocketAddr,env};

pub struct ServerConfig{
    pub host: String,
    pub port: u16,
    pub addr: SocketAddr,
    pub jwt_secret: String,
    pub exp: u64
}

impl ServerConfig{
    pub fn load()->Result<Self,String>{
        

        let host = env::var("SERVER_HOST")
            .map_err(|_|format!("SERVER_HOST environment variable is not set"))?;

        let port: u16 = env::var("SERVER_PORT")
            .map_err(|_|format!("SERVER_PORT environment variable is not set"))?
            .parse()
            .map_err(|_|format!("SERVER_PORT environment variable is not a valid u16 number"))?;

        let addr: SocketAddr = format!("{}:{}", host, port)
            .parse()
            .map_err(|_|"Invalid SERVER_HOST and PORT combination")?;

            
            

        let jwt_secret = env::var("JWT_SECRET")
            .map_err(|_|"JWT_SECRET environment variable missing")?;

        let exp = env::var("DURATION")
            .map_err(|_|"DURATION Missing")?
            .parse::<u64>()
            .map_err(|e|format!("{}",e))?;
      
    
        Ok(ServerConfig{
            host,
            port,
            addr,
            jwt_secret,
            exp
        })
    }
}
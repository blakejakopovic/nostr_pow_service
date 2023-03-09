use clap::Parser;
use std::net::SocketAddr;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct AppArgs {
   #[arg(long, env="LISTEN", default_value="127.0.0.1:3030")]
   pub socket_addr: SocketAddr,

   #[arg(long, env="RELAY_IDENTIFIER", default_value="ws://127.0.0.1")]
   pub relay_identifier: String,

   #[arg(long, env="PUBKEY_WHITELIST", default_value="", value_delimiter=',')]
   pub pubkey_whitelist: Vec<String>,

   #[arg(long, env="MIN_POW_DIFFICULTY", default_value="10")]
   pub min_pow_difficulty: u16,

   #[arg(long, env="MAX_POW_DIFFICULTY", default_value="25")]
   pub max_pow_difficulty: u16,
}

pub struct AppConfig {
    pub relay_identifier: String,
    pub pubkey_whitelist: Vec<String>,
    pub min_pow_difficulty: u16,
    pub max_pow_difficulty: u16,
}

impl AppConfig {
  pub fn new(
    relay_identifier: String,
    pubkey_whitelist: Vec<String>,
    min_pow_difficulty: u16,
    max_pow_difficulty: u16
  ) -> Self {

    Self {
        relay_identifier,
        pubkey_whitelist,
        min_pow_difficulty,
        max_pow_difficulty,
    }
  }
}

#[macro_use]
extern crate log;
use anyhow::Result;
use clap::Parser;
use dotenv::dotenv;
use nostrgraph_pow_service::config::{AppArgs, AppConfig};
use nostrgraph_pow_service::websocket::ws_connect;
use serde_json::json;
use std::sync::Arc;
use warp::Filter;
use warp_real_ip::real_ip;


#[tokio::main]
async fn main() -> Result<()> {

    env_logger::init();

    dotenv().ok();

    let args = AppArgs::parse();

    let app_config = Arc::new(AppConfig::new(
        args.relay_identifier,
        args.pubkey_whitelist,
        args.min_pow_difficulty,
        args.max_pow_difficulty
    ));

    let app_config_warp = warp::any().map(move || Arc::clone(&app_config));

    // This allows us get an real source IP Address if behind Nginx
    let proxy_addr = [127, 0, 0, 1].into();
    let real_ip_warp = warp::any()
        .and(real_ip(vec![proxy_addr]));

    // https://github.com/nostr-protocol/nips/blob/master/11.md
    let server_info = json!({
        "name": "Nostr PoW Service Provider",
        "description": "Nostr Proof of Work Service Provider",
        // "pubkey": "",
        // "contact": "",
        // "supported_nips": [], // TODO: add NIP-XX once we have a number
        "software": "Nostr PoW Service",
        "version": "Infinite"
    });
    let server_info_route = warp::path::end()
      .and(warp::header::exact("ACCEPT", "application/nostr+json"))
      .map(move || {
          debug!("Request for server info");
          warp::reply::json(&server_info)
      });

    let websocket_route = warp::path::end()
        .and(warp::ws())
        .and(app_config_warp)
        .and(real_ip_warp)
        .map(|ws: warp::ws::Ws, app_config, real_ip|
            ws.on_upgrade(move |socket|
                ws_connect(socket, app_config, real_ip)
            )
        );

    let routes = server_info_route.or(websocket_route);

    println!("Starting server: {}", args.socket_addr);
    warp::serve(routes).run(args.socket_addr).await;

    Ok(())
}

use anyhow::Result;
use crate::commands::{NostrMessage, AuthCmd, PowCmd};
use crate::config::AppConfig;
use crate::NEXT_USERID;
use crate::payment::{debt_account, credit_account, payment_required};
use crate::peer::PeerInfo;
use crate::pow::{generate_pow, validate_pow_request};
use futures::{StreamExt, SinkExt};
use std::net::IpAddr;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tokio::sync::{RwLock, mpsc, mpsc::error::SendTimeoutError};
use tokio::time::Duration;
use warp::ws::{Message, WebSocket};

const MPSC_SEND_TIMEOUT: Duration = Duration::from_millis(20);


pub async fn ws_connect(ws: WebSocket, app_config: Arc<AppConfig>, real_ip: Option<IpAddr>) {

    let peer_id = NEXT_USERID.fetch_add(1, Ordering::Relaxed);
    info!("New connection: {peer_id} from {real_ip:?}");

    let peer_info = Arc::new(RwLock::new(PeerInfo::new(peer_id, real_ip)));

    // Split websocket connection
    let (mut ws_tx, mut ws_rx) = ws.split();

    // Peer websocket outbox (peer will drop messages if they aren't taking after this limit)
    let (peer_tx, mut peer_rx) = mpsc::channel::<Message>(100);

    // Send AUTH request
    let auth_req_str = peer_info.read().await.generate_auth_request_cmd();
    let event_msg = Message::text(auth_req_str);
    if let Err(e) = ws_tx.send(event_msg).await {
        error!("Writing AUTH request to websocket failed. Closing connection: {e:?}");

        // Close websocket
        return
    }

    'run_loop: loop {

        let peer_info = Arc::clone(&peer_info);
        let peer_tx = peer_tx.clone();

        tokio::select! {

            // Handle peer inbound (inbox) message
            result = ws_rx.next() => {

                trace!("ws_rx.next(): {:?}", result);

                match result {
                    None => {
                        debug!("Broadcast ws_rx has None. Disconnecting peer");
                        break 'run_loop;
                    },

                    Some(Err(err)) => {
                        // Protocol(ResetWithoutClosingHandshake)
                        // Io(Os { code: 32, kind: BrokenPipe, message: "Broken pipe" }):
                        debug!("{err:?}:");
                        break 'run_loop;
                    },

                    Some(Ok(result)) => {

                        if result.is_close() {
                            break 'run_loop;
                        }

                        // TODO: Currently we only return Ok(()). Add bad peer protection here
                        if let Err(e) = handle_rx_message(Arc::clone(&peer_info), peer_tx, result, Arc::clone(&app_config)).await {
                            info!("Disconnecting peer: {:?} - {e:?}", peer_info.read().await.real_ip);
                            break 'run_loop;
                        }
                    },
                }
            },

            // Handle peer outbound (outbox) message
            Some(result) = peer_rx.recv() => {

                trace!("peer_rx.recv(): {result:?}");

                if let Err(e) = ws_tx.send(result.clone()).await {
                    debug!("Writing to websocket failed. Closing connection: {e:?}");
                    break 'run_loop
                }

                trace!("Sent message to peer: {result:?}");
            },
        }
    }
}

// Handle peer worker inbox
async fn handle_rx_message(peer_info: Arc<RwLock<PeerInfo>>,
                           peer_tx: mpsc::Sender<Message>,
                           msg: Message,
                           app_config: Arc<AppConfig>,
    ) -> Result<()> {

    if let Ok(msg) = msg.to_str() {

        debug!("Received message: {msg:?}");

        // Parse JSON
        let nostr_msg: Result<NostrMessage> = serde_json::from_str(msg).map_err(std::convert::Into::into);

        match nostr_msg {
            Err(err) => {
                debug!("Unable to parse message: {msg}: {err:?}");
                send_notice(peer_tx, "Unable to parse message").await;
            },

            Ok(NostrMessage::AuthMsg(auth_msg)) => {
                info!("AUTH Message: {auth_msg:?}");
                handle_auth_msg(app_config, peer_info, auth_msg, peer_tx).await?;
            },

            Ok(NostrMessage::PowMsg(pow_msg)) => {
                info!("POW Message: {pow_msg:?}");
                handle_pow_msg(app_config, peer_info, pow_msg, peer_tx).await?;
            },
        }
    }

    Ok(())
}

async fn handle_auth_msg(
        app_config: Arc<AppConfig>,
        peer_info: Arc<RwLock<PeerInfo>>,
        auth_msg: AuthCmd,
        peer_tx: mpsc::Sender<Message>
    ) -> Result<()> {

    let auth_event = auth_msg.event;

    let mut peer_info = peer_info.write().await;

    match peer_info.check_auth_response(app_config.relay_identifier.clone(), &auth_event) {
        Err(e) => {
            let notice_msg = format!("Invalid AUTH response for challenge: {} - {e:?}", peer_info.auth_challenge);
            send_notice(peer_tx, &notice_msg).await
        },
        Ok(_) => {
            let notice_msg = format!("Authorised: {}", &auth_event.pubkey);
            send_notice(peer_tx, &notice_msg).await
        }
    }

    Ok(())
}

async fn handle_pow_msg(
        app_config: Arc<AppConfig>,
        peer_info: Arc<RwLock<PeerInfo>>,
        pow_msg: PowCmd,
        peer_tx: mpsc::Sender<Message>
    ) -> Result<()> {

    let peer_info = peer_info.read().await;

    if let false = peer_info.auth_confirmed {
        send_notice(peer_tx, "restricted: you need to authorise to confirm your pubkey first").await;
        return Ok(())
    }

    let authenticated_pubkey = peer_info.pubkey.clone().unwrap_or_default();

    if let Err(e) = validate_pow_request(
                        app_config.min_pow_difficulty,
                        app_config.max_pow_difficulty,
                        pow_msg.target_pow,
                        &pow_msg.event,
                        &authenticated_pubkey
        ) {

        send_notice(peer_tx, &format!("pow: invalid pow request: {e:?}")).await;
        return Ok(())
    }

    let payment_required = payment_required(&app_config.pubkey_whitelist, authenticated_pubkey);

    // TODO: Need to add logging / record keeping here for financial and troubleshooting

    if payment_required == true {
        if let Err(_e) = debt_account(&pow_msg.event.pubkey, pow_msg.target_pow, &pow_msg.event.id).await {
            // TODO: Need to add logging / record keeping here for financial and troubleshooting
            send_notice(peer_tx, &format!("pow: out of credit")).await;
            return Ok(())
        }
    }

    info!("Generating target POW: {} for {:?}", &pow_msg.target_pow, &pow_msg.event);

    match generate_pow(pow_msg.target_pow, pow_msg.event.clone()).await {
        Err(e) => {

            warn!("generate_pow failed. {} {} {} {e:?}", &pow_msg.event.pubkey, pow_msg.target_pow, &pow_msg.event.id);

            if payment_required == true {
                match credit_account(&pow_msg.event.pubkey, pow_msg.target_pow, &pow_msg.event.id).await {
                    Ok(_) => {
                        // TODO: Need to add logging / record keeping here for financial and troubleshooting
                    },
                    Err(_) => {
                        // TODO: Need to add logging / record keeping here for financial and troubleshooting
                    },
                }
            }

            send_notice(peer_tx, &format!("pow: request failed")).await;
            return Ok(())
        },

        Ok(event) => {
            let event_json_str = serde_json::to_string(&event)?;
            let reply_str = format!(r#"["POW",{}]"#, &event_json_str);
            send_msg(peer_tx, &reply_str).await;

            return Ok(())
        },
    }
}

async fn send_msg(peer_tx: mpsc::Sender<Message>, message: &str) {
    let notice_msg = Message::text(message);

    if let Err(SendTimeoutError::Closed(e)) = peer_tx.send_timeout(notice_msg, MPSC_SEND_TIMEOUT).await {
        debug!("Writing to websocket failed. Close client?: {:?}", e);
    }
}

async fn send_notice(peer_tx: mpsc::Sender<Message>, notice: &str) {
    let notice_str = format!(r#"["NOTICE","{notice}"]"#);
    let notice_msg = Message::text(notice_str);

    if let Err(SendTimeoutError::Closed(e)) = peer_tx.send_timeout(notice_msg, MPSC_SEND_TIMEOUT).await {
        debug!("Writing to websocket failed. Close client?: {:?}", e);
    }
}

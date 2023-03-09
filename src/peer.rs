use anyhow::{anyhow,Result};
use crate::{get_timestamp, get_event_first_tag_with_value};
use nostr_rs_relay::event::Event;
use std::net::IpAddr;
use uuid::Uuid;

const AUTH_CREATED_AT_DELTA_SEC: u64 = 300; // 5 minutes

#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub id: usize,
    pub real_ip: Option<IpAddr>,
    pub auth_challenge: String,
    pub auth_confirmed: bool,
    pub pubkey: Option<String>,
}

impl PeerInfo {
    pub fn new(id: usize, real_ip: Option<IpAddr>) -> Self {

        let auth_challenge = Uuid::new_v4().to_string();

        Self {
          id,
          real_ip,
          auth_challenge,
          auth_confirmed: false,
          pubkey: None
        }
    }

    pub fn generate_auth_request_cmd(&self) -> String {
        format!(r#"["AUTH", "{}"]"#, self.auth_challenge)
    }

    pub fn check_auth_response(&mut self, relay_identifier: String, event: &Event) -> Result<()> {

        // Ensure event is valid
        event.validate()?;

        // Ensure event kind is 22242
        if event.kind != 22242 {
            return Err(anyhow!("Invalid event kind"));
        }

        // Ensure event created_at is reasonable (within 5 minutes)
        let now = get_timestamp();
        if !(now-AUTH_CREATED_AT_DELTA_SEC..=now+AUTH_CREATED_AT_DELTA_SEC).contains(&event.created_at) {
            return Err(anyhow!("Invalid event created_at. Must be within 5 minutes now"));
        }

        // Ensure relay tag matches
        if Some(relay_identifier) != get_event_first_tag_with_value(&event, "relay") {
            return Err(anyhow!("Invalid relay tag"));
        }

        // Ensure challenge matches
        if Some(self.auth_challenge.to_string()) != get_event_first_tag_with_value(&event, "challenge") {
            return Err(anyhow!("Invalid challenge tag"));
        }

        // Otherwise, we are all good!
        self.auth_confirmed = true;
        self.pubkey = Some(event.pubkey.to_string());

        debug!("AUTHENTICATED: {:?}", self.pubkey);

        Ok(())
    }
}

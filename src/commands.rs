use anyhow::{anyhow,Result};
use nostr_rs_relay::event::Event;
use serde::{Deserialize, Serialize};

/// Supported Nostr Commands
#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, Debug)]
#[serde(untagged)]
pub enum NostrMessage {
    AuthMsg(AuthCmd),
    PowMsg(PowCmd),
}

/// ["AUTH", {AUTH_EVENT}]
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct AuthCmd {
    pub cmd: String,
    pub event: Event,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct Auth {
    pub event: Event,
}

impl From<AuthCmd> for Result<Auth> {
    fn from(msg: AuthCmd) -> Result<Auth> {
        if msg.cmd == "AUTH" {
            Ok(Auth { event: msg.event })
        } else {
            Err(anyhow!("Unknown command"))
        }
    }
}

/// ["POW", TARGET_POW, {POW_EVENT}, PUBLISH]
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct PowCmd {
    pub cmd: String,
    pub target_pow: u16,
    pub event: Event,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct Pow {
    pub target_pow: u16,
    pub event: Event,
}

impl From<PowCmd> for Result<Pow> {
    fn from(msg: PowCmd) -> Result<Pow> {
        if msg.cmd == "POW" {
            Ok(Pow { target_pow: msg.target_pow, event: msg.event })
        } else {
            Err(anyhow!("Unknown command"))
        }
    }
}

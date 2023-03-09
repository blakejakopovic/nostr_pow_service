#[macro_use]
extern crate log;

pub mod commands;
pub mod config;
pub mod payment;
pub mod peer;
pub mod pow;
pub mod websocket;

use nostr_rs_relay::event::Event;
use std::sync::atomic::AtomicUsize;
use std::time::{SystemTime, UNIX_EPOCH};

pub const CREATED_AT_DELTA_SEC: u64 = 600;

pub static NEXT_USERID: AtomicUsize = AtomicUsize::new(1);

pub fn get_timestamp() -> u64 {
    let now = SystemTime::now();
    let since_the_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
    since_the_epoch.as_secs()
}

fn get_event_first_tag_with_value(event: &Event, tag: &str) -> Option<String> {
    event.tags
        .iter()
        .find(|t| t.get(0).map_or(false, |c| c.to_lowercase() == tag.to_lowercase()))
        .map(|t| t[1].to_string())
}

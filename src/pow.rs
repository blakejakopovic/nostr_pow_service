use anyhow::{anyhow,Result};
use crate::CREATED_AT_DELTA_SEC;
use crate::get_timestamp;
use nostr_rs_relay::event::Event;
use nostr_rust::events::EventPrepare;
use rand::Rng;
use serde_json::json;
use tokio::task;
use tokio::time::Instant;

pub fn get_digest_input(event: &EventPrepare) -> String {
    json!([
        0,
        event.pub_key,
        event.created_at,
        event.kind,
        event.tags,
        event.content
    ])
    .to_string()
}

pub fn get_content_id(event: &EventPrepare) -> String {
    sha256::digest(get_digest_input(event))
}

pub fn count_leading_zero_bits(content_id: Vec<u8>) -> u16 {
    let mut total: u16 = 0;

    for c in content_id {
        let bits = c.leading_zeros() as u16;
        total += bits;
        if bits != 8 {
            break;
        }
    }
    total
}

pub async fn generate_pow(target_difficulty: u16, mut event: Event) -> Result<Event> {

    // Generate event payload
    let event_prepare = EventPrepare {
        pub_key: event.pubkey.clone(),
        created_at: event.created_at,
        kind: event.kind as u16,
        tags: event.tags.clone(),
        content: event.content.clone(),
    };

    // Use spawn_blocking to offload to a new thread
    let compute = task::spawn_blocking(move || {
        generate_pow_event(event_prepare, target_difficulty).unwrap()
    });

    match compute.await {
        Err(e) => {
            error!("generate_pow failed with error: {e:?}");
            return Err(anyhow!("Event Proof of Work failed"))
        },

        Ok((event_id, nonce, leading_zeros)) => {

            // TODO: Refactor this, but we need to make Event and EventPrepare work or make our own
            event.id = event_id;
            event.tags.push(nonce);

            info!("Target Difficulty: {target_difficulty}. Found: {leading_zeros}");

            // TODO: Can/should we remove it entirely?
            event.sig = "".to_owned();
        }
    };


    Ok(event)
}

pub fn generate_pow_event(mut event: EventPrepare, difficulty: u16) -> Result<(String, Vec<String>, u16)> {
    let mut rng = rand::thread_rng();

    let start = Instant::now();
    let mut attempts = 0;
    loop {

        // We need to set created_at on first loop for temporal spam-protection
        // Note: If we don't bump timestamp, we may run out of nonce options
        event.created_at = get_timestamp();

        let nonce: u32 = rng.gen_range(0..u32::MAX);

        let nonce_tag = vec![
            "nonce".to_string(),
            nonce.to_string(),
            difficulty.to_string(),
        ];

        event.tags.push(nonce_tag.clone());

        let content_id = get_content_id(&event);
        let content_id_hex = hex::decode(&content_id)?;

        let leading_zeros = count_leading_zero_bits(content_id_hex);
        if leading_zeros >= difficulty {

            let total_duration = Instant::now().duration_since(start);

            // POW 25 Request - found 25 with 19109387 attempts in 737417 ms (Macbook Pro)
            info!("POW {difficulty} Request - found {leading_zeros} with {attempts} attempts in {} ms", total_duration.as_millis());
            return Ok((content_id, nonce_tag, leading_zeros))
        }

        // Remove failed nonce tag
        event.tags.pop();

        attempts += 1;
    }
}

pub fn validate_pow_request(min_pow: u16, max_pow: u16, target_difficulty: u16, event: &Event, request_pubkey: &str) -> Result<()> {

    info!("{event:?}");

    // Validate target_difficulty is between min and max
    if !(min_pow..=max_pow).contains(&target_difficulty) {
        return Err(anyhow!("restricted: target difficulty must be between {min_pow} and {max_pow}"))
    }

    // Validate signature (prevent impersonation and validate the pubkey)
    if let Err(_) = event.validate() {
        return Err(anyhow!("error: invalid input event"))
    }

    // Check request event POW matches authorised pubkey
    if event.pubkey != request_pubkey {
        return Err(anyhow!("error: event pubkey doesn't match authenticated pubkey"))
    }

    // Check event created_at is reasonable (within 15 minutes)
    let now = get_timestamp();
    if !(now-CREATED_AT_DELTA_SEC..=now+CREATED_AT_DELTA_SEC).contains(&event.created_at) {
        return Err(anyhow!("Invalid event created_at. Must be within 5 minutes now"));
    }

    Ok(())
}

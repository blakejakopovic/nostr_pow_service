use anyhow::Result;
use nostrgraph_pow_service::pow::generate_pow;
use nostr_rs_relay::event::Event;
use tokio::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {

    let event = Event {
        id: "0".to_owned(),
        pubkey: "0".to_owned(),
        delegated_by: None,
        created_at: 0,
        kind: 0,
        tags: vec![],
        content: "".to_owned(),
        sig: "0".to_owned(),
        tagidx: None,
    };

    for difficulty in 10..=25 {

        let start = Instant::now();
        let iterations = 10;

        for _ in 1..=iterations {
          generate_pow(difficulty, event.clone()).await?;
        }
        let duration = Instant::now().duration_since(start).as_millis();

        println!("Generated {iterations} in {} ms @ POW {difficulty}", duration);
    }

    Ok(())
}

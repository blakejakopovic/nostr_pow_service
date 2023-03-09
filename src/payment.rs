use anyhow::{anyhow, Result};

fn price_for_difficulty(target_difficulty: u16) -> u32 {

    // Each leading bit in difficulty effectively doubles the computational cost
    // This pricing model, while not doubling, seems like a fair starting point

    // PoW sats
    // 10  3
    // 11  5
    // 12  7
    // 13  10
    // 14  15
    // 15  23
    // 16  34
    // 17  50
    // 18  75
    // 19  112
    // 20  169
    // 21  255
    // 22  387
    // 23  588
    // 24  898
    // 25  1375
    // 26  2112
    // 27  3257
    // 28  5037
    // 29  7817
    // 30  12167

    // =(TARGET-7)^(TARGET/10)
    ((target_difficulty-7) as f32).powf(target_difficulty as f32 /10.0) as u32
}

pub fn payment_required(whitelist: &Vec<String>, pubkey: String) -> bool {
    !whitelist.contains(&pubkey)
}

pub async fn debt_account(pubkey: &str, difficulty: u16, source_event_id: &str) -> Result<()> {
    // Verify that pubkey has funds (and deduct?)
    let price_sat = price_for_difficulty(difficulty);
    info!("Request cost {} satoshi for {difficulty} target difficulty", price_sat);

    // Connect to accounts and check account balance and deduct amount
    info!("Deducting {price_sat} from {pubkey}'s account");
    let payment_response: Result<()> = Ok(());

    if let Err(_e) = payment_response { //payment_response(&event.pubkey, price_sat) {

        // TODO: Need to add logging / record keeping here for financial and troubleshooting
        return Err(anyhow!("{}: insuffecient funds", source_event_id))
    }

    // TODO: Need to add logging / record keeping here for financial and troubleshooting

    Ok(())
}

pub async fn credit_account(pubkey: &str, difficulty: u16, source_event_id: &str) -> Result<()> {
    info!("Credit account request for: {pubkey}: difficulty {difficulty} for {source_event_id}");

    let _price_sat = price_for_difficulty(difficulty);

    async {}.await;

    // TODO: Need to add logging / record keeping here for financial and troubleshooting

    Ok(())
}

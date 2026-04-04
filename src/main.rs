extern crate chrono;

use chrono::Local;
use reqwest;
use serde::Deserialize;
// use std::collections::HashMap;
use tokio;

fn make_url(leverancier: i32) -> String {
    let date = Local::now();
    let date_string = date.format("%Y-%m-%d");
    format!(
        "https://www.stroomperuur.nl/ajax/tarieven.php?leverancier={}&datum={}",
        leverancier, date_string
    )
}

// async fn get_data() -> json::Result<String> {
//     reqwest::get("https://www.rust-lang.org")
//         .await?
//         .text()
//         .await?;
// }

#[derive(Deserialize, Debug)]
struct PricingData(Vec<f32>, Vec<f32>, f32, f32);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let url = make_url(2);

    let resp = client.get(url).send().await?.json::<PricingData>().await?;
    println!("{resp:#?}");
    Ok(())
}

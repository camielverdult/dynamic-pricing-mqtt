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
        "https://www.stroomperuur.nl/ajax/tarieven.php?leverancier={}&datum={}&kwartier=1`",
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
struct PricingDataResponse {
    purchase_price: Vec<f64>,
    taxes: Vec<f64>,
    average_purchase_price: f64,
    purchasing_fee: f64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let url = make_url(2);

    println!("URL: {url}");

    let resp = client
        .get(url)
        .send()
        .await?
        .json::<PricingDataResponse>()
        .await?;
    println!("{resp:#?}");
    Ok(())
}

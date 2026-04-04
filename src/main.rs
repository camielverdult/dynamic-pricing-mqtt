extern crate chrono;

use chrono::{DateTime, Local};
use reqwest;
use serde::Deserialize;
use tokio;

#[derive(Deserialize, Debug)]
struct PricingDataResponse {
    purchase_price: Vec<f64>,
    taxes: Vec<f64>,
    average_purchase_price: f64,
    purchasing_fee: f64,
}

#[derive(Debug)]
struct PricingData {
    date: DateTime<Local>,
    pricings: PricingDataResponse,
}

async fn get_data(client: reqwest::Client) -> Result<PricingData, reqwest::Error> {
    let leverancier = 2;

    let date = Local::now();
    let date_string = date.format("%Y-%m-%d");

    let url = format!(
        "https://www.stroomperuur.nl/ajax/tarieven.php?leverancier={}&datum={}&kwartier=1`",
        leverancier, date_string
    );

    let resp = client
        .get(url)
        .send()
        .await?
        .json::<PricingDataResponse>()
        .await;

    Ok(PricingData {
        date: date,
        pricings: resp?,
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let data = get_data(client).await?;

    println!("{:#?}", data);
    Ok(())
}

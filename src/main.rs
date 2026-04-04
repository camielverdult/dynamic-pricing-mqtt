extern crate chrono;

use chrono::Local;
use reqwest;
use serde::Deserialize;
use tokio;

fn make_url(leverancier: i32) -> String {
    let date = Local::now();
    let date_string = date.format("%Y-%m-%d");
    format!(
        "https://www.stroomperuur.nl/ajax/tarieven.php?leverancier={}&datum={}&kwartier=1`",
        leverancier, date_string
    )
}

async fn get_data(client: reqwest::Client) -> Result<PricingDataResponse, reqwest::Error> {
    let url = make_url(2);

    let resp = client
        .get(url)
        .send()
        .await?
        .json::<PricingDataResponse>()
        .await?;

    Ok(resp)
}

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

    let data = get_data(client).await.unwrap();

    println!("{}", data.average_purchase_price);
    Ok(())
}

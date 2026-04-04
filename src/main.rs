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

enum Leverancier {
    Generic = 0,
    All_in_power = 4,
    ANWB_Energie = 3,
    Budget_Energie = 15,
    Coolblue_Energie = 10,
    Delta_Energie = 22,
    easyEnergy = 5,
    Eneco = 17,
    Energie_VanOns = 6,
    Energiedirect = 16,
    Energiek = 21,
    EnergyZero = 7,
    Engie = 23,
    Essent = 20,
    Frank_Energie = 8,
    GroeneStroomLokaal = 9,
    NextEnergy = 11,
    Oxxio = 19,
    Tibber = 1,
    Vandebron = 14,
    Vattenfall = 18,
    Vrijopnaam = 12,
    Zonneplan = 2,
}

async fn get_data(
    client: reqwest::Client,
    leverancier: Leverancier,
) -> Result<PricingData, reqwest::Error> {
    let date = Local::now();
    let date_string = date.format("%Y-%m-%d");

    let url = format!(
        "https://www.stroomperuur.nl/ajax/tarieven.php?leverancier={}&datum={}&kwartier=1`",
        leverancier as i32, date_string
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

    let data = get_data(client, Leverancier::Zonneplan).await?;

    println!("{:#?}", data);
    Ok(())
}

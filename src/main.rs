extern crate chrono;

use chrono::{DateTime, Local, Timelike};
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
    BudgetEnergie = 15,
    CoolblueEnergie = 10,
    DeltaEnergie = 22,
    easyEnergy = 5,
    Eneco = 17,
    EnergieVanOns = 6,
    Energiedirect = 16,
    Energiek = 21,
    EnergyZero = 7,
    Engie = 23,
    Essent = 20,
    FrankEnergie = 8,
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

fn get_price_at_time(prices: &PricingDataResponse, time: DateTime<Local>) -> Option<f64> {
    let hour = time.hour(); // 0-23
    let minute = time.minute(); // 0-59

    // Integer division by 15 gives us 0, 1, 2, or 3!
    let quarter = minute / 15;

    let index = (hour * 4 + quarter) as usize;

    assert!(prices.purchase_price.len() >= index);
    assert!(prices.taxes.len() >= index);

    let inkoop_prijs = prices.purchase_price.get(index).copied()?;

    let inkoop_vergoeding = prices.purchasing_fee;
    let taxes = prices.taxes.get(index).copied()?;

    Some(inkoop_prijs + inkoop_vergoeding + taxes)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let data = get_data(client, Leverancier::Zonneplan).await?;

    let price_now = get_price_at_time(&data.pricings, Local::now());

    println!("{}", price_now.unwrap());

    Ok(())
}

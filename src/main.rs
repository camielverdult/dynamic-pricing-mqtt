extern crate chrono;

use chrono::{Date, DateTime, Datelike, Local, Timelike};
use reqwest;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use serde::Deserialize;
use std::error::Error;
use std::time::Duration;
use tokio;
use tokio::{task, time};

#[derive(Deserialize, Debug)]
struct PricingDataResponse {
    purchase_price: Vec<f32>,
    taxes: Vec<f32>,
    average_purchase_price: f32,
    purchasing_fee: f32,
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
    client: &reqwest::Client,
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

fn index_at_time(time: DateTime<Local>) -> u32 {
    let hour = time.hour(); // 0-23
    let minute = time.minute(); // 0-59

    // Integer division by 15 gives us 0, 1, 2, or 3!
    let quarter = minute / 15;

    let index = (hour * 4 + quarter) as u32;

    index
}

fn time_for_index(i: u32) -> DateTime<Local> {
    let hour = (i / 4) as u32;
    let quarter = (i % 4) as u32;
    let minute = quarter * 15;

    Local::now()
        .with_hour(hour)
        .unwrap()
        .with_minute(minute)
        .unwrap()
        .with_second(0)
        .unwrap()
        .with_nanosecond(0)
        .unwrap()
}

fn get_price_at_time(prices: &PricingDataResponse, time: DateTime<Local>) -> Option<f32> {
    let index = index_at_time(time) as usize;

    assert!(prices.purchase_price.len() >= index);
    assert!(prices.taxes.len() >= index);

    let inkoop_prijs = prices.purchase_price.get(index).copied()?;

    let inkoop_vergoeding = prices.purchasing_fee;
    let taxes = prices.taxes.get(index).copied()?;

    let cent_price = inkoop_prijs + inkoop_vergoeding + taxes;

    Some(cent_price)
}

#[tokio::main]
async fn main() {
    let req_client = reqwest::Client::new();

    let mut mqttoptions = MqttOptions::new("rumqtt-async", "test.mosquitto.org", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (mqtt_client, mut _eventloop) = AsyncClient::new(mqttoptions, 10);
    mqtt_client
        .subscribe("energy_price/now", QoS::AtMostOnce)
        .await
        .unwrap();

    loop {
        let data = get_data(&req_client, Leverancier::Zonneplan).await.unwrap();

        let now = Local::now();
        let index = index_at_time(now);

        while index < 24 * 4 {
            let price_now = get_price_at_time(&data.pricings, now);

            println!("{}", price_now.unwrap());

            // mqtt_client
            //     .publish("hello/rumqtt", QoS::AtLeastOnce, false, vec![i; i as usize])
            //     .await
            //     .unwrap();

            let time_at_next_index = time_for_index(index);
            let time_until_next = time_at_next_index - Local::now();
            let ms = time_until_next.num_milliseconds().abs();

            println!("Sleeping {} ms", ms);

            time::sleep(Duration::from_millis(ms.try_into().unwrap())).await;
        }
    }

    // Ok(())
}

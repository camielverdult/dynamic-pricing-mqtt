extern crate chrono;

use chrono::{DateTime, Datelike, Local, Timelike};
use reqwest;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use serde::Deserialize;
use std::time::Duration;
use tokio;
use tokio::time;

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

#[derive(Clone, Copy)]
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
    leverancier: &Leverancier,
) -> Result<PricingData, reqwest::Error> {
    let date = Local::now();
    let date_string = date.format("%Y-%m-%d");

    let url = format!(
        "https://www.stroomperuur.nl/ajax/tarieven.php?leverancier={}&datum={}&kwartier=1`",
        *leverancier as u8, date_string
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

fn index_at_time(time: DateTime<Local>) -> i32 {
    let hour = time.hour(); // 0-23
    let minute = time.minute(); // 0-59

    // Integer division by 15 gives us 0, 1, 2, or 3!
    let quarter = minute / 15;

    let index = (hour * 4 + quarter) as i32;

    index
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

    let (mqtt_client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    // Spawn a background task to constantly poll the event loop
    tokio::spawn(async move {
        loop {
            // This actually drives the connection, sends packets, and receives ACKs
            if let Err(e) = eventloop.poll().await {
                println!("MQTT connection error: {:?}", e);
                // Backoff a bit before retrying the poll on error
                time::sleep(Duration::from_secs(1)).await;
            }
        }
    });

    let leverancier = Leverancier::Zonneplan;

    let last_data_fetched = chrono::Local::now() - chrono::Duration::days(1);

    // initialise data without any value, we will fetch it in the loop
    let mut data = get_data(&req_client, &leverancier).await.unwrap();

    loop {
        let now = Local::now();

        if last_data_fetched.day() != now.day() {
            data = get_data(&req_client, &leverancier).await.unwrap();
        }

        let price_now = get_price_at_time(&data.pricings, now).unwrap();

        println!("{}:{} = €{}", now.hour(), now.minute(), price_now);

        mqtt_client
            .publish(
                "energy_price/now",
                QoS::AtLeastOnce,
                false,
                price_now.to_string(),
            )
            .await
            .unwrap();

        // Calculate time to sleep until minute value is incremented by 1 and seconds are 0
        let next_minute = now + chrono::Duration::minutes(1);
        let next_minute = next_minute
            .with_second(0)
            .unwrap()
            .with_nanosecond(0)
            .unwrap();
        let time_until_next = next_minute - now;

        let ms = time_until_next.num_milliseconds().abs();
        let sleep_time = Duration::from_millis(ms.try_into().unwrap());
        // let sleep_time = Duration::from_millis(1000);

        println!("Sleeping {} ms", ms);

        time::sleep(sleep_time).await;
    }
}

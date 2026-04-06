# Dynamic pricing MQTT

Usage:
```bash
cargo build --release

# For testing purposes, you can use the public MQTT broker at test.mosquitto.org.
# In production, you should use your own MQTT broker and set the MQTT_HOST environment variable accordingly.
export MQTT_HOST="test.mosquitto.org"

# Use Zonneplan as the supplier, this determines the purchasing fee used in the dynamic pricing calculation.
# In production, you should set this to your actual supplier.
export LEVERANCIER="Zonneplan"

% ./target/release/dynamic-pricing-mqtt
Starting with config: Config {
    timezone: Europe/Amsterdam,
    host: "test.mosquitto.org",
    port: 1883,
    username: "",
    password: "",
    leverancier: Zonneplan,
}
Sending discovery payload:HaDiscoveryConfig {
    topic: "homeassistant/device/2/config",
    payload: HaDiscoveryPayload {
        dev: HaDevice {
            ids: "dynamic_energy_price_01",
            name: "Dynamic Energy Pricing",
            mf: "Camiel",
            mdl: "MQTT Bridge",
            sw: "1.0",
            sn: "dyn_price_01",
            hw: "1.0",
        },
        o: HaOrigin {
            name: "dynamic-pricing-mqtt",
            sw: "1.0",
            url: "https://github.com/camielverdult/dynamic-pricing-mqtt",
        },
        cmps: {
            "dynamic_energy_price": HaComponent {
                p: "sensor",
                name: "Current Energy Price",
                device_class: "monetary",
                unit_of_measurement: "EUR",
                value_template: "{{ value_json.price }}",
                unique_id: "energy_price_now_01",
                suggested_display_precision: 5,
            },
        },
        state_topic: "dynamic_energy_price/now",
        qos: 1,
    },
}
15:14 = €0.0523
Sending payload "{\"price\": 0.0523}"
Sleeping 8410 ms
15:15 = €0.064399995
Sending payload "{\"price\": 0.064399995}"
Sleeping 54996 ms
```

## Docker

To build the image:
```sh
docker build -t dynamic-pricing-mqtt .
```

To run the image:
```sh
docker run -d \
     --name dynamic-pricing \
     -e MQTT_HOST="0.0.0.0" \
     -e LEVERANCIER="Zonneplan" \
     dynamic-pricing-mqtt
```

There are several environment variables you can set:

`MQTT_HOST`: "0.0.0.0" by default
`MQTT_PORT`: 1883 by default
`MQTT_USERNAME`: empty string ("") by default
`MQTT_PASSWORD`: empty string ("") by default
`LEVERANCIER`: "Generic" by default, can be one of the following: `Generic`, `All_in_power`, `ANWB_Energie`, `BudgetEnergie`, `CoolblueEnergie`, `DeltaEnergie`, `easyEnergy`, `Eneco`, `EnergieVanOns`, `Energiedirect`, `Energiek`, `EnergyZero`, `Engie`, `Essent`, `FrankEnergie`, `GroeneStroomLokaal`, `NextEnergy`, `Oxxio`, `Tibber`, `Vandebron`, `Vattenfall`, `Vrijopnaam`, `Zonneplan`

TODO:
- [x] Integration with home assistant (https://www.home-assistant.io/integrations/mqtt/)
- [x] Make a docker image for easy deployment

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
    topic: "dynamic_energy_price",
}
17:39 = €0.13069
Sleeping 25310 ms
17:40 = €0.13069
Sleeping 59999 ms
```

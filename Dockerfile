# Stage 1: Build environment
FROM rust:1-slim-bookworm AS builder

WORKDIR /usr/src/app

# Install dependencies required to compile some Rust crates (like OpenSSL for reqwest)
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy over your manifests
COPY Cargo.toml Cargo.lock ./

# Create a dummy src/main.rs to cache dependencies
# This prevents Docker from re-downloading all crates every time you change a line of code.
RUN mkdir src && echo "fn main() {}" > src/main.rs && echo "" > src/lib.rs


RUN cargo build --release
RUN rm -rf src

# 3. Copy your actual source code
COPY src ./src

# 4. Update the timestamp on main.rs to force Cargo to rebuild it, then compile the release binary
RUN touch src/main.rs && cargo build --release


# Stage 2: Production environment
FROM debian:bookworm-slim

WORKDIR /app

# Install CA certificates so reqwest can successfully verify HTTPS connections
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/dynamic-pricing-mqtt /usr/local/bin/dynamic-pricing-mqtt

# Set default environment variables (these can be overridden when running the container)
ENV TIMEZONE="Europe/Amsterdam"
ENV LEVERANCIER="Generic"
ENV MQTT_HOST="127.0.0.1"
ENV MQTT_PORT="1883"
ENV MQTT_USERNAME=""
ENV MQTT_PASSWORD=""

# Run the binary
CMD ["dynamic-pricing-mqtt"]

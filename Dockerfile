# HA passes BUILD_FROM; standalone builds use the default
ARG BUILD_FROM=debian:bookworm-slim

# Build stage
FROM rust:1-bookworm AS builder

WORKDIR /app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src target/release/deps/gym*

# Copy source and build
COPY . .
RUN cargo build --release

# Runtime stage
FROM ${BUILD_FROM}

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libsqlite3-0 \
    jq \
    openssl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/gym /app/gym
COPY --from=builder /app/static /app/static
COPY run.sh /app/run.sh
RUN chmod +x /app/run.sh

RUN mkdir -p /data

ENV HOST=0.0.0.0
ENV PORT=3000

EXPOSE 3000

# Home Assistant add-on labels
LABEL \
    io.hass.name="Gym Tracker" \
    io.hass.description="Personal gym workout tracker" \
    io.hass.type="addon" \
    io.hass.version="1.0.0"

CMD ["/app/run.sh"]

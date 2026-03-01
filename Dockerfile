# HA passes BUILD_FROM; standalone builds use the default
ARG BUILD_FROM=alpine:3.19

# Build stage
FROM rust:1-alpine AS builder

RUN apk add --no-cache musl-dev sqlite-dev pkgconf

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

RUN apk add --no-cache ca-certificates sqlite-libs jq openssl bash

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

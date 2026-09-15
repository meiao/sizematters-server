# syntax=docker/dockerfile:1

########## Builder ##########
FROM rust:1-bookworm AS builder

# Frontend build tooling: wasm target + trunk (bundles the Leptos/WASM app)
RUN rustup target add wasm32-unknown-unknown \
    && cargo install trunk --locked

WORKDIR /app
COPY . .

# Build the WASM frontend. frontend/Trunk.toml points its output at ../dist,
# which is what the server serves as static files.
RUN cd frontend && trunk build --release

# Build the server binary (workspace member, so `-p` builds just what it needs)
RUN cargo build --release -p sizematters-server

########## Runtime ##########
FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/sizematters-server ./sizematters-server
COPY --from=builder /app/dist ./dist

ENV RUST_LOG=actix_server=info,actix_web=info
EXPOSE 8080

CMD ["./sizematters-server"]

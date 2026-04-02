FROM rust:1.77-slim AS builder

WORKDIR /app

# Cache dependency build layer separately
COPY Cargo.toml ./
RUN mkdir src && echo "fn main(){}" > src/main.rs && echo "" > src/lib.rs
RUN cargo build --release 2>/dev/null || true
RUN rm src/main.rs src/lib.rs

COPY src ./src
RUN touch src/main.rs src/lib.rs && cargo build --release

# ---- runtime image ----
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates libsqlite3-0 && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/supplement-buddy .

EXPOSE 8001

CMD ["./supplement-buddy"]

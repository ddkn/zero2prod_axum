FROM lukemathwalker/cargo-chef:latest-rust-1.79 AS chef
# Switch to the /app directory, analagous to `mkdir -p /app; cd /app`
WORKDIR /app
RUN apt update && apt install lld clang -y

# Build recipe
FROM chef AS planner
COPY . . 
RUN cargo chef prepare --recipe-path recipe.json

# Compile code based on cached packages
FROM chef AS builder
COPY --from=planner /app/recipe.json .
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release

# Build small image
FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt update -y \
  && apt install -y --no-install-recommends openssl ca-certificates \
  && apt autoremove -y \
  && apt clean -y \
  && rm -rf /var/lib/apt/lists*
COPY --from=builder /app/target/release/zero2prod_axum zero2prod_axum
COPY --from=builder /app/settings.production.toml .
ENV APP_ENV=production
ENTRYPOINT ["./zero2prod_axum"]

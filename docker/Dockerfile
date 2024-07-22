FROM rust:1.79 AS builder

# Switch to the /app directory, analagous to `mkdir -p /app; cd /app`
WORKDIR /app
RUN apt update && apt install lld clang -y
# Copy all files from working environment to Docker image
COPY . . 
ENV SQLX_OFFLINE=true
RUN cargo check --release
RUN cargo build --release

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

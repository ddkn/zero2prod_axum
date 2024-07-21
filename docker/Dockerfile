FROM rust:1.79

# Switch to the /app directory, analagous to `mkdir -p /app; cd /app`
WORKDIR /app
RUN apt update && apt install lld clang -y
# Copy all files from working environment to Docker image
COPY . . 
ENV SQLX_OFFLINE true
RUN cargo check --release
RUN cargo build --release

ENTRYPOINT ["./target/release/zero2prod_axum"]

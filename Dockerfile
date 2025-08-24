FROM rust:bookworm AS builder

RUN apt update && apt-get update
RUN apt upgrade -y && apt-get upgrade -y
RUN apt install -y \
    build-essential \
    curl \
    neovim \
    fish \
    pkg-config \
    libssl-dev

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build -j8

FROM debian:bookworm-slim

RUN apt update && apt-get update
RUN apt upgrade -y && apt-get upgrade -y
RUN apt install -y \
    bash \
    curl \
    fish \
    xz-utils \
    neovim \
    ca-certificates \
    libssl3 \
    gnupg

RUN curl -fsSL https://pgp.mongodb.com/server-7.0.asc | gpg -o /usr/share/keyrings/mongodb-server-7.0.gpg --dearmor
RUN echo "deb [ signed-by=/usr/share/keyrings/mongodb-server-7.0.gpg ] http://repo.mongodb.org/apt/debian bookworm/mongodb-org/7.0 main" | \
    tee /etc/apt/sources.list.d/mongodb-org-7.0.list
RUN apt-get update
RUN apt-get install -y mongodb-mongosh mongodb-database-tools

WORKDIR /app

COPY --from=builder /app/target/debug/apiodactyl .
COPY ./docker-entrypoint.sh .
COPY ./Rocket.toml .
RUN chmod +x docker-entrypoint.sh

ENTRYPOINT ["./docker-entrypoint.sh"]

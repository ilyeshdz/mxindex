FROM rust:latest

RUN apt-get update && apt-get install -y \
    libpq-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY diesel.toml ./
COPY src ./src
COPY migrations ./migrations

RUN cargo build --release

EXPOSE 8000

CMD ["cargo", "run", "--release"]
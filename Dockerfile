FROM rust:latest as cargo-build

WORKDIR /usr/src/pushy

RUN apt-get update && apt-get install -y postgresql-client

COPY Cargo.toml Cargo.toml

RUN mkdir src/

RUN echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs

RUN cargo build --release

RUN rm -f target/release/deps/pushy*

COPY . .

RUN cargo build --release

RUN cargo install --path .

CMD ["/usr/local/cargo/bin/pushy"]
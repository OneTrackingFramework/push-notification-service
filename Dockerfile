FROM rust:latest

WORKDIR /usr/src/pushy

COPY Cargo.toml Cargo.toml
COPY ./src ./src

RUN rm -f target/release/deps/pushy*
RUN cargo build --release
RUN cargo install --path .

CMD ["/usr/local/cargo/bin/pushy"]
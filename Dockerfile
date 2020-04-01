FROM rust:latest

WORKDIR /usr/src/pushy

RUN rm src/*.rs
COPY ./src ./src
COPY Cargo.toml Cargo.toml

RUN rm -f target/release/deps/pushy*
RUN cargo build --release
RUN cargo install --path .

CMD ["/usr/local/cargo/bin/pushy"]
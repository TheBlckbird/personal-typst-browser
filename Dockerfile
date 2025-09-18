FROM rust:1-bullseye

WORKDIR /app

COPY . ./

RUN cargo install typst-cli --locked
RUN cargo build --release

CMD ["./target/release/personal_typst_browser"]

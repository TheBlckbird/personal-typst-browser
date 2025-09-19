FROM rust:1-bullseye

WORKDIR /app

COPY . ./

RUN cargo install typst-cli --locked
RUN cargo build --release

LABEL org.opencontainers.image.source=https://github.com/TheBlckbird/personal-typst-browser
LABEL org.opencontainers.image.description="Browse your personal typst files"
LABEL org.opencontainers.image.licenses=MIT

CMD ["./target/release/personal_typst_browser"]

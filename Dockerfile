FROM rust:1.89.0-alpine3.22 AS chef
ENV RUSTFLAGS=-Dwarnings
WORKDIR /build
RUN apk add --no-cache build-base && cargo install cargo-chef


FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json


FROM chef AS cooker
COPY --from=planner /build/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json --target x86_64-unknown-linux-musl


FROM cooker AS builder
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl


FROM alpine:3.22
RUN adduser -u 1000 -D user
RUN apk add --no-cache git jinja2-cli openssh-client
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/templater /templater
USER user
ENTRYPOINT ["/templater"]

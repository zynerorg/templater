ARG BASE_IMAGE=scratch

FROM rust:1.85-alpine3.21 AS build

WORKDIR /build
RUN apk add --no-cache build-base
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl


FROM ${BASE_IMAGE}

COPY --from=build /build/target/x86_64-unknown-linux-musl/release/templater /templater
ENTRYPOINT ["/templater"]

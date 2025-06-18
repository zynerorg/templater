FROM rust:1.87.0-alpine3.22 AS build

WORKDIR /build
RUN apk add --no-cache build-base
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl


FROM alpine:3.22

RUN adduser -u 1000 -D user
RUN apk add --no-cache git jinja2-cli openssh-client

COPY --from=build /build/target/x86_64-unknown-linux-musl/release/templater /templater

USER user
ENTRYPOINT ["/templater"]

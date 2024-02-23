FROM rust:1.76-slim-bullseye as build

WORKDIR /build
COPY /src /build/src
COPY Cargo.toml /build/

RUN cargo build --release

FROM debian:bullseye-slim

COPY --from=build /build/target/release/kube-nimble /kube-nimble

ENTRYPOINT ["/kube-nimble"]

FROM rust:slim as builder

RUN mkdir -p /build/monitor_bot

COPY . /build/monitor_bot

WORKDIR /build/monitor_bot

RUN cargo build --release

FROM gcr.io/distroless/cc-debian12

COPY --from=builder /build/monitor_bot/target/release/monitor_bot /monitor_bot

ENTRYPOINT ["/monitor_bot"]

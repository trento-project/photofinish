FROM rust:1.59 as builder

WORKDIR /home/photofinish/

COPY . .
RUN cargo build --release

FROM gcr.io/distroless/cc

WORKDIR /home/photofinish/
COPY --from=builder /home/photofinish/target/release/photofinish .
WORKDIR /data
VOLUME ["/data"]
ENTRYPOINT ["/home/photofinish/photofinish"]

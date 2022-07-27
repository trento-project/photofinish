FROM rust:1.59 as builder

WORKDIR /home/photofinish/

COPY . .
RUN cargo build --release

FROM registry.suse.com/bci/rust:latest

WORKDIR /home/photofinish/
COPY --from=builder /home/photofinish/target/release/photofinish .
WORKDIR /data
VOLUME ["/data"]
ENTRYPOINT ["/home/photofinish/photofinish"]

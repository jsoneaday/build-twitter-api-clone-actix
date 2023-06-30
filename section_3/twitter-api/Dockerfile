FROM rust:1.68.2 as builder
WORKDIR /usr/src/chatterserver
COPY . .
RUN cargo install --debug --path .
 
FROM debian:bullseye
RUN apt-get update
RUN apt-get install -y wget
RUN apt-get install -y build-essential
COPY --from=builder /usr/src/chatterserver /usr/local/bin/chatterserver
WORKDIR /usr/local/bin/chatterserver
ENTRYPOINT [ "./target/debug/server-rs" ]
FROM rust:1.68.2 as builder
WORKDIR /usr/src/chatterserver
COPY . .
RUN cargo install --debug --path .
 
FROM debian:buster-slim
RUN apt-get update
RUN apt-get install -y wget
RUN apt-get install -y build-essential
RUN apt-get install -y zlib1g-dev
ARG OPENSSL_VERSION=1.1.0g
RUN wget https://www.openssl.org/source/openssl-${OPENSSL_VERSION}.tar.gz
RUN tar xvfz openssl-${OPENSSL_VERSION}.tar.gz
RUN cd openssl-${OPENSSL_VERSION} && ./config && make && make install
RUN echo '/usr/local/lib' >> /etc/ld.so.conf
RUN cat /etc/ld.so.conf
RUN ldconfig
RUN echo 'export LD_LIBRARY_PATH=/usr/local/lib' >> ~/.bash_profile && . ~/.bash_profile
RUN openssl version
COPY --from=builder /usr/src/chatterserver /usr/local/bin/chatterserver
WORKDIR /usr/local/bin/chatterserver
ENTRYPOINT [ "./target/debug/server-rs" ]
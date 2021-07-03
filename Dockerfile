# syntax=docker/dockerfile-upstream:master-labs
FROM rust:1.53-alpine as builder
RUN <<eot
  rustup component add rustfmt
  apk add --upgrade --no-cache \
    build-base \
    ca-certificates \
    openssl-dev \
    protoc tzdata
eot
WORKDIR /usr/src/myblog-api
COPY . .
RUN <<eot
  cargo build --release
eot

FROM scratch
COPY --from=builder /etc/ssl/certs/* /etc/ssl/certs/
COPY --from=builder /usr/share/zoneinfo/* /usr/share/zoneinfo/
COPY --from=builder /usr/src/myblog-api/target/release/myblog-api /usr/local/bin/myblog-api
ENTRYPOINT ["myblog-api"]
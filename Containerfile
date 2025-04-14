FROM docker.io/library/rust:alpine as builder

WORKDIR /usr/src/frcc
COPY . .

RUN apk add --no-cache -U musl-dev openssl-dev font-noto
RUN cargo build --release

FROM docker.io/library/alpine:latest

COPY --from=builder /usr/src/frcc/target/release/frcc /usr/local/bin/frcc

CMD ["/usr/local/bin/frcc"]

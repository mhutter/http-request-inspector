FROM docker.io/library/rust:1.86-alpine AS build
WORKDIR /app

RUN apk add --no-cache mold musl-dev
COPY . .
RUN cargo build --release --locked

FROM docker.io/library/alpine:latest
ENV LISTEN_ADDR=0.0.0.0:8080
EXPOSE 8080
ENTRYPOINT []
CMD ["/usr/local/bin/http-request-inspector"]
RUN apk add --no-cache curl
COPY --from=build /app/target/release/http-request-inspector /usr/local/bin
USER 101010

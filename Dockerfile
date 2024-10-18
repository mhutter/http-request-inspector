FROM docker.io/library/rust:1.82.0-slim-bookworm AS build
WORKDIR /app

COPY Cargo.toml Cargo.lock .
COPY src src/
RUN cargo build --release --locked

FROM docker.io/library/debian:bookworm-slim
ENV LISTEN_ADDR=0.0.0.0:8080
EXPOSE 8080
ENTRYPOINT []
CMD ["/usr/local/bin/http-request-inspector"]
COPY --from=build /app/target/release/http-request-inspector /usr/local/bin
USER 101010

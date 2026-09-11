FROM rust:1-bookworm AS builder
WORKDIR /src
COPY . .
RUN cargo build --release -p sdkwork-api-forum-standalone-gateway

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /src/target/release/sdkwork-api-forum-standalone-gateway /usr/local/bin/sdkwork-api-forum-standalone-gateway
ENV SDKWORK_FORUM_APPLICATION_PUBLIC_INGRESS_BIND=0.0.0.0:8080
EXPOSE 8080
CMD ["sdkwork-api-forum-standalone-gateway"]

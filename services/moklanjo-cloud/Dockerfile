# Moklanjo Cloud API image (App Runner / ECS). Separate from Khaynjo.
FROM rust:bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock* ./
COPY migrations ./migrations
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=build /src/target/release/moklanjo-cloud /app/moklanjo-cloud
COPY migrations ./migrations
ENV HOST=0.0.0.0
ENV PORT=8080
EXPOSE 8080
CMD ["/app/moklanjo-cloud"]

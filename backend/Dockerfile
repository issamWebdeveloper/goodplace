# ---- Build ----
FROM rust:1-bookworm AS build
WORKDIR /app
# Pré-compilation des dépendances (couche mise en cache)
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && cargo build --release && rm -rf src
COPY . .
RUN touch src/main.rs && cargo build --release

# ---- Exécution ----
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* && useradd --system --uid 10001 goodplace
COPY --from=build /app/target/release/goodplace-api /usr/local/bin/goodplace-api
USER goodplace
ENV BIND_ADDR=0.0.0.0:8080
EXPOSE 8080
CMD ["goodplace-api"]

# planer
FROM lukemathwalker/cargo-chef:latest-rust-1.69-slim-bookworm as planer
WORKDIR /app
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

# builder
FROM lukemathwalker/cargo-chef:latest-rust-1.69-slim-bookworm as builder
COPY --from=planer /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release

# runtime
FROM debian:bullseye-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
  && apt-get install -y --no-install-recommends openssl ca-certificates \
  # Clean up
  && apt-get autoremove -y \
  && apt-get clean -y \
  && rm -rf /var/lib/apt/lists/*
COPY --from=builder ./target/release/zero2prod zero2prod
COPY configuration configuration
ENV APP_ENVIRONMENT production
ENTRYPOINT [ "./zero2prod" ]

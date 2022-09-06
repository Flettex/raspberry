FROM lukemathwalker/cargo-chef:latest-rust-bullseye AS chef
WORKDIR /raspberry

# Literally so dumb I have to set up SSL
RUN apt-get install -y --reinstall ca-certificates && \
    mkdir /usr/local/share/ca-certificates/cacert.org && \
    wget -P /usr/local/share/ca-certificates/cacert.org http://www.cacert.org/certs/root.crt http://www.cacert.org/certs/class3.crt && \
    update-ca-certificates && \
    git config --global http.sslCAinfo /etc/ssl/certs/ca-certificates.crt

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
ARG DATABASE_URL
ARG EMAIL_PASSWORD
ARG RAILWAY_STATIC_URL
# Docker being a dumb dumb and can't access production env variables during build time (which sqlx and my macros use unfortunately)
ENV DATABASE_URL=$DATABASE_URL
ENV EMAIL_PASSWORD=$EMAIL_PASSWORD
ENV RAILWAY_STATIC_URL = $RAILWAY_STATIC_URL
COPY --from=planner /raspberry/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin raspberry-backend-app

FROM debian:bullseye-slim AS runtime
WORKDIR /raspberry
COPY --from=builder /raspberry/target/release/raspberry-backend-app /usr/local/bin
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/raspberry-backend-app"]
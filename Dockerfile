# Build rust executables
FROM rust:1.84 as chef
RUN cargo install cargo-chef
WORKDIR /usr/src/app


FROM chef as planner
COPY src/ src/
COPY Cargo.lock Cargo.toml .
RUN cargo chef prepare --recipe-path recipe.json


FROM chef as builder
COPY --from=planner /usr/src/app/recipe.json recipe.json
# build dependencies
RUN cargo chef cook --release --recipe-path recipe.json

# build application
COPY src/ src/
COPY .sqlx/ .sqlx/
COPY Cargo.lock Cargo.toml .
RUN SQLX_OFFLINE=true cargo build --release

# Generate static content for reverse proxy
COPY templates/ templates/
COPY styles/ styles/
COPY blog/ blog/
RUN /usr/src/app/target/release/static-build


FROM nginx:alpine as reverse-proxy

ARG NGINX_CONF=nginx/nginx.prod.conf

COPY ${NGINX_CONF} /etc/nginx/nginx.conf
COPY nginx/locations.conf /etc/nginx/locations.conf
RUN rm /etc/nginx/conf.d/default.conf

COPY --from=builder  /usr/src/app/build /build
RUN apk add --no-cache curl ca-certificates  # Needed for healthcheck

EXPOSE 80


FROM debian:bookworm-slim as blog-post-dispatcher

RUN apt-get update && apt-get install -y git
COPY .git/ .git/
COPY --from=builder /usr/src/app/target/release/blog-post-dispatcher /usr/local/bin/

CMD ["/usr/local/bin/blog-post-dispatcher"]


FROM debian:bookworm-slim as backend

COPY templates/ templates/
COPY blog/ blog/
COPY --from=builder /usr/src/app/build build/
COPY --from=builder /usr/src/app/target/release/dynamic-site /usr/local/bin/
RUN apt-get update && apt-get install -y curl ca-certificates # Needed for healthcheck

CMD ["/usr/local/bin/dynamic-site"]

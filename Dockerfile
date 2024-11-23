# Build rust executable
FROM rust:1.78 as chef
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
## build dynamic
RUN cargo build --bin dynamic-site --release
COPY templates/ templates/
COPY styles/ styles/
COPY src/ src/
COPY Cargo.lock Cargo.toml .
## build static
RUN cargo run --bin static-build --release

# Set up server
FROM nginx as runtime

## prepare static content and rust binary
COPY templates/ templates/
COPY styles/ styles/
COPY --from=builder  /usr/src/app/build /build
COPY --from=builder /usr/src/app/target/release/dynamic-site /usr/local/bin/

# prepare config and static content for nginx
COPY nginx.conf /etc/nginx/nginx.conf
RUN rm /etc/nginx/conf.d/default.conf

## prepare supervisord for nginx and rust processes
RUN apt-get update && apt-get install -y supervisor
COPY supervisord.conf /etc/supervisor/conf.d/supervisord.conf

EXPOSE 80
CMD ["/usr/bin/supervisord", "-c", "/etc/supervisor/conf.d/supervisord.conf"]

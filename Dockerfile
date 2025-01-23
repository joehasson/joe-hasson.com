# Build rust executable
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
## build dynamic
COPY src/ src/
# Copy the sqlx-data.json file if you're using the offline mode
COPY .sqlx/ .sqlx/
COPY Cargo.lock Cargo.toml .
RUN cargo build --bin dynamic-site --release
## build static as well
COPY templates/ templates/
COPY styles/ styles/
COPY blog/ blog/
RUN cargo run --bin static-build --release

# Set up server
FROM nginx as runtime

## prepare static content and rust binary
COPY templates/ templates/
COPY styles/ styles/
COPY blog/ /blog
COPY --from=builder  /usr/src/app/build /build
COPY --from=builder /usr/src/app/target/release/dynamic-site /usr/local/bin/

# prepare config
COPY nginx.conf /etc/nginx/nginx.conf
RUN rm /etc/nginx/conf.d/default.conf

## prepare supervisord for nginx and rust processes
RUN apt-get update && apt-get install -y supervisor
COPY supervisord.conf /etc/supervisor/conf.d/supervisord.conf

EXPOSE 80
CMD ["/usr/bin/supervisord", "-c", "/etc/supervisor/conf.d/supervisord.conf"]

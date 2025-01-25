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
COPY src/ src/
COPY .sqlx/ .sqlx/
COPY Cargo.lock Cargo.toml .
RUN SQLX_OFFLINE=true cargo build --release

# Generate static content
COPY templates/ templates/
COPY styles/ styles/
COPY blog/ blog/
RUN /usr/src/app/target/release/static-build

# Set up server
FROM nginx as runtime

## prepare static content and rust binaries
COPY templates/ templates/
COPY styles/ styles/
COPY blog/ blog/
COPY .git/ .git/
COPY --from=builder  /usr/src/app/build /build
COPY --from=builder /usr/src/app/target/release/dynamic-site /usr/local/bin/
COPY --from=builder /usr/src/app/target/release/blog-post-dispatcher /usr/local/bin/


# prepare config
COPY nginx.conf /etc/nginx/nginx.conf
RUN rm /etc/nginx/conf.d/default.conf

## prepare supervisord for nginx and rust processes
RUN apt-get update
RUN apt-get install -y supervisor
COPY supervisord.conf /etc/supervisor/conf.d/supervisord.conf

# Instal git for blog-post-dispatcher
RUN apt-get install -y git

EXPOSE 80
CMD [\
    "/bin/sh", "-c", \
    "/usr/local/bin/blog-post-dispatcher && \
    /usr/bin/supervisord -c /etc/supervisor/conf.d/supervisord.conf"\
]

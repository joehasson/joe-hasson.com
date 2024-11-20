# Build rust executable
FROM rust:1.76 as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release
RUN /usr/src/app/target/release/static-build

# Set up server
FROM nginx

## prepare static content and rust binary
COPY --from=builder  /usr/src/app/build /static
COPY --from=builder /usr/src/app/target/release/dynamic-site /usr/local/bin/

# prepare config and static content for nginx
COPY nginx.conf /etc/nginx/nginx.conf
RUN rm /etc/nginx/conf.d/default.conf

## prepare supervisord for nginx and rust processes
RUN apt-get update && apt-get install -y supervisor
COPY supervisord.conf /etc/supervisor/conf.d/supervisord.conf

EXPOSE 80
CMD ["/usr/bin/supervisord", "-c", "/etc/supervisor/conf.d/supervisord.conf"]

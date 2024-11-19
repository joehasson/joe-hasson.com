# Build rust executable
FROM rust:1.76 as builder
WORKDIR /usr/src/app
COPY dynamic_site .
RUN cargo build --release

# Set up server
FROM nginx

## prepare rust binary
COPY --from=builder /usr/src/app/target/release/dynamic-site /usr/local/bin/

# prepare config and static content for nginx
COPY nginx.conf /etc/nginx/nginx.conf
RUN rm /etc/nginx/conf.d/default.conf
COPY static_site/build /static

## prepare supervisord for nginx and rust processes
RUN apt-get update && apt-get install -y supervisor
COPY supervisord.conf /etc/supervisor/conf.d/supervisord.conf

EXPOSE 80
CMD ["/usr/bin/supervisord", "-c", "/etc/supervisor/conf.d/supervisord.conf"]

# syntax=docker/dockerfile:1
FROM rust:1.99.0-alpine AS build
WORKDIR /app
RUN apk add --no-cache musl-dev
COPY . /app
RUN cargo install --locked --path .

FROM docker:latest
RUN apk add --no-cache git
COPY --from=build /usr/local/cargo/bin/hooked /usr/local/bin/hooked
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh
WORKDIR /app
ENTRYPOINT ["/entrypoint.sh"]
CMD ["hooked"]

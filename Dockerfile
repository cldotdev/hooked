# syntax=docker/dockerfile:1
FROM rust:1.99.0-alpine AS build
WORKDIR /app
RUN apk add --no-cache musl-dev
COPY . /app
RUN cargo install --locked --path .

FROM docker:latest
RUN apk add --no-cache git && \
    git config --global credential.helper store && \
    git config --global url."https://github.com/".insteadOf "ssh://git@github.com/" && \
    git config --global --add url."https://github.com/".insteadOf "git@github.com:" && \
    git config --global --add url."https://github.com/".insteadOf "git+ssh://git@github.com/"
COPY --from=build /usr/local/cargo/bin/hooked /usr/local/bin/hooked
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh
WORKDIR /app
ENTRYPOINT ["/entrypoint.sh"]
CMD ["hooked"]

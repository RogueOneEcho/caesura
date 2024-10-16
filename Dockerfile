# Build imdl binary
FROM rust:alpine AS imdl
RUN apk add --no-cache curl
RUN curl "https://github.com/casey/intermodal/releases/download/v0.1.14/imdl-v0.1.14-x86_64-unknown-linux-musl.tar.gz" \
      --location \
      --show-error \
      --silent \
    | tar \
      --extract \
      --gzip \
      --directory "/bin" \
      --file - \
      "imdl"

# Build caesura binary
FROM rust:alpine AS builder
RUN apk add --no-cache libc-dev cargo-edit
# Build just the dependencies with version 0.0.0 so they're cached
WORKDIR /app
COPY Cargo.toml Cargo.lock build.rs /app
RUN mkdir -p src && echo 'fn main() {}' > /app/src/main.rs
RUN cargo fetch
RUN cargo build --release --locked
# Set the version
COPY . /app
ARG VERSION=0.0.0
RUN cargo set-version $VERSION
# Build the release binary
RUN cargo build --release

# Build final image with minimal dependencies
FROM alpine:latest
RUN apk add --no-cache flac lame sox imagemagick imagemagick-jpeg
COPY --from=imdl /bin/imdl /bin/imdl
COPY --from=builder /app/target/release/caesura /bin/caesura
WORKDIR /
ENTRYPOINT ["caesura"]

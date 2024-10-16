# Build imdl binary
FROM rust:alpine AS imdl
RUN apk add --no-cache libc-dev
RUN cargo install imdl --version 0.1.14

# Build cargo-edit binary
FROM rust:alpine AS cargo-edit
RUN apk add --no-cache libc-dev
RUN cargo install cargo-edit --version 0.13.0

# Build caesura binary
FROM rust:alpine AS builder
RUN apk add --no-cache libc-dev
# Build just the dependencies with version 0.0.0 so they're cached
WORKDIR /app
COPY Cargo.toml Cargo.lock build.rs /app
RUN mkdir -p src && echo 'fn main() {}' > /app/src/main.rs
RUN cargo fetch
RUN cargo build --release --locked
# Set the version
COPY --from=cargo-edit /usr/local/cargo/bin/cargo-set-version /bin/cargo-set-version
COPY . /app
ARG VERSION=0.0.0
RUN cargo set-version $VERSION
# Build the release binary
RUN cargo build --release

# Build final image with minimal dependencies
FROM alpine:latest
RUN apk add --no-cache flac lame sox imagemagick imagemagick-jpeg
COPY --from=imdl /usr/local/cargo/bin/imdl /bin/imdl
COPY --from=builder /app/target/release/caesura /bin/caesura
WORKDIR /
ENTRYPOINT ["caesura"]

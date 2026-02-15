# Build FLAC from source
# https://github.com/xiph/flac#building-with-gnu-autotools
FROM alpine:latest AS flac-builder
RUN apk add --no-cache build-base libogg-dev curl
ARG FLAC_VERSION=1.5.0
RUN curl -L "https://downloads.xiph.org/releases/flac/flac-${FLAC_VERSION}.tar.xz" \
      --show-error \
      --silent \
    | tar --extract --xz \
    && cd "flac-${FLAC_VERSION}" \
    && ./configure \
      --prefix=/usr \
      --disable-static \
      --disable-thorough-tests \
    && make -j"$(nproc)" \
    && make install DESTDIR=/flac-install

# Cargo chef base
FROM rust:alpine AS chef
RUN apk add --no-cache libc-dev && cargo install cargo-chef cargo-edit cargo-auditable
WORKDIR /app

# Prepare recipe
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Build caesura binary
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
ARG VERSION=0.0.0
RUN cargo set-version -p caesura $VERSION
RUN cargo auditable build --release

# Build final image with minimal dependencies
FROM alpine:latest
RUN apk add --no-cache libogg lame sox
COPY --from=flac-builder /flac-install/usr/bin/flac /usr/bin/flac
COPY --from=flac-builder /flac-install/usr/lib/libFLAC.so* /usr/lib/
COPY --from=builder /app/target/release/caesura /bin/caesura
ENV CAESURA_DOCKER=1
WORKDIR /
ENTRYPOINT ["caesura"]

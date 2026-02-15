# Build FLAC from source
# https://github.com/xiph/flac#building-with-gnu-autotools
FROM alpine:latest AS flac-builder
RUN apk add --no-cache build-base libogg-dev curl
ARG FLAC_VERSION=1.5.0
ARG FLAC_SHA256=f2c1c76592a82ffff8413ba3c4a1299b6c7ab06c734dee03fd88630485c2b920
RUN curl -L "https://downloads.xiph.org/releases/flac/flac-${FLAC_VERSION}.tar.xz" \
      --show-error --silent -o flac.tar.xz \
    && echo "${FLAC_SHA256}  flac.tar.xz" | sha256sum -c - \
    && tar xf flac.tar.xz \
    && cd "flac-${FLAC_VERSION}" \
    && ./configure \
      --prefix=/usr \
      --disable-static \
      --disable-thorough-tests \
    && make -j"$(nproc)" \
    && make install DESTDIR=/flac-install

# Build sox-ng from source
# https://codeberg.org/sox_ng/sox_ng
FROM alpine:latest AS sox-ng-builder
RUN apk add --no-cache build-base autoconf automake \
    libpng-dev fftw-dev libogg-dev
COPY --from=flac-builder /flac-install/usr/lib/ /usr/lib/
COPY --from=flac-builder /flac-install/usr/include/ /usr/include/
ARG SOX_NG_VERSION=14.7.0.7
ARG SOX_NG_SHA256=c494658ef29ebe84eddf525fcdcfe7ba67fca3ee778402cf46f1ec1178086b61
RUN wget -q "https://codeberg.org/sox_ng/sox_ng/releases/download/sox_ng-${SOX_NG_VERSION}/sox_ng-${SOX_NG_VERSION}.tar.gz" \
    && echo "${SOX_NG_SHA256}  sox_ng-${SOX_NG_VERSION}.tar.gz" | sha256sum -c - \
    && tar xf "sox_ng-${SOX_NG_VERSION}.tar.gz" \
    && cd "sox_ng-${SOX_NG_VERSION}" \
    && ./configure --prefix=/usr --disable-static --without-libltdl --disable-openmp --without-sndfile \
    && make -j"$(nproc)" \
    && make install DESTDIR=/sox-ng-install

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

# Dev target for running tests
FROM builder AS dev
RUN apk add --no-cache lame libogg libpng fftw
COPY --from=flac-builder /flac-install/usr/bin/flac /usr/bin/flac
COPY --from=flac-builder /flac-install/usr/bin/metaflac /usr/bin/metaflac
COPY --from=flac-builder /flac-install/usr/lib/libFLAC.so* /usr/lib/
COPY --from=sox-ng-builder /sox-ng-install/usr/bin/sox_ng /usr/bin/sox_ng
COPY --from=sox-ng-builder /sox-ng-install/usr/lib/libsox_ng.so* /usr/lib/
ENV CAESURA_DOCKER=1
RUN cargo build --release --tests --all-features
ENTRYPOINT ["cargo"]
CMD ["test", "--release", "--all-features"]

# Build final image with minimal dependencies
FROM alpine:latest
RUN apk add --no-cache libogg lame libpng fftw
COPY --from=flac-builder /flac-install/usr/bin/flac /usr/bin/flac
COPY --from=flac-builder /flac-install/usr/lib/libFLAC.so* /usr/lib/
COPY --from=sox-ng-builder /sox-ng-install/usr/bin/sox_ng /usr/bin/sox_ng
COPY --from=sox-ng-builder /sox-ng-install/usr/lib/libsox_ng.so* /usr/lib/
COPY --from=builder /app/target/release/caesura /bin/caesura
ENV CAESURA_DOCKER=1
WORKDIR /
ENTRYPOINT ["caesura"]

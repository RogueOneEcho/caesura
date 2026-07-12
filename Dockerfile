# Build FLAC from source
# https://github.com/xiph/flac#building-with-gnu-autotools
FROM alpine:latest AS flac
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
    && make install DESTDIR=/artifacts

# Build SoX_ng from source
# https://codeberg.org/sox_ng/sox_ng
FROM alpine:latest AS sox
RUN apk add --no-cache build-base autoconf automake \
    libpng-dev fftw-dev libogg-dev
COPY --from=flac /artifacts/usr/lib/ /usr/lib/
COPY --from=flac /artifacts/usr/include/ /usr/include/
ARG SOX_NG_VERSION=14.8.0.1
ARG SOX_NG_SHA256=7698a1b2699499b0b38fa95a15bb56c68928d97b144bce03b7ecb76fe9c46698
RUN wget -q "https://codeberg.org/sox_ng/sox_ng/releases/download/sox_ng-${SOX_NG_VERSION}/sox_ng-${SOX_NG_VERSION}.tar.gz" \
    && echo "${SOX_NG_SHA256}  sox_ng-${SOX_NG_VERSION}.tar.gz" | sha256sum -c - \
    && tar xf "sox_ng-${SOX_NG_VERSION}.tar.gz" \
    && cd "sox_ng-${SOX_NG_VERSION}" \
    && ./configure --prefix=/usr --disable-static --without-libltdl --disable-openmp --without-sndfile \
    && make -j"$(nproc)" \
    && make install DESTDIR=/artifacts

# Build LAME from source
# https://lame.sourceforge.io
FROM alpine:latest AS lame
RUN apk add --no-cache build-base curl pkgconf
ARG LAME_VERSION=4.0
ARG LAME_SHA256=3df5124d5ad3a98312ffd7ba6a9b36230e4f8a3e66d3ce0f425e336c32d216eb
RUN curl -L "https://downloads.sourceforge.net/project/lame/lame/${LAME_VERSION}/lame-${LAME_VERSION}.tar.gz" \
      --show-error --silent -o lame.tar.gz \
    && echo "${LAME_SHA256}  lame.tar.gz" | sha256sum -c - \
    && tar xf lame.tar.gz \
    && cd "lame-${LAME_VERSION}" \
    # LAME 4.0 ships an inconsistent frontend: lame.h defines
    # DEPRECATED_OR_OBSOLETE_CODE_REMOVED=1, dropping the id3tag_set_*_ucs2
    # declarations, but frontend/parse.c still calls them in its unused --utf8
    # tagging path. GCC 14+ (Alpine) makes implicit-declaration and
    # incompatible-pointer-type fatal; downgrade them to warnings. The functions
    # remain in libmp3lame so linking succeeds, and caesura only encodes so this
    # path is never reached.
    && export CFLAGS="-Wno-error=implicit-function-declaration -Wno-error=incompatible-pointer-types" \
    && ./configure \
      --prefix=/usr \
      --disable-static \
      --disable-decoder \
    && make -j"$(nproc)" \
    && make install DESTDIR=/artifacts

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
RUN cargo chef cook --release --locked --recipe-path recipe.json
COPY . .
ARG VERSION=0.0.0
RUN cargo set-version -p caesura $VERSION
RUN cargo auditable build --release --locked

# Dev target for running tests
FROM builder AS dev
RUN apk add --no-cache libogg libpng fftw
COPY --from=flac /artifacts/usr/bin/flac /usr/bin/flac
COPY --from=flac /artifacts/usr/bin/metaflac /usr/bin/metaflac
COPY --from=flac /artifacts/usr/lib/libFLAC.so* /usr/lib/
COPY --from=lame /artifacts/usr/bin/lame /usr/bin/lame
COPY --from=lame /artifacts/usr/lib/libmp3lame.so* /usr/lib/
COPY --from=sox /artifacts/usr/bin/sox_ng /usr/bin/sox_ng
COPY --from=sox /artifacts/usr/lib/libsox_ng.so* /usr/lib/
ENV CAESURA_DOCKER=1
RUN cargo build --release --locked --tests --all-features
ENTRYPOINT ["cargo"]
CMD ["test", "--release", "--all-features"]

# Hardened runtime image
# Follows the Docker Hardened Images (DHI) approach:
# https://github.com/docker-hardened-images/catalog/blob/e2579891e42131f738018407d8bbe0c66379cb8b/image/alpine-base/alpine-3.23/3.23.yaml
# - Remove apk so the package manager cannot be used to install attack tools
# - Remove docs and man pages to reduce image size
# - Run as non-root user (65532 matches the DHI nonroot convention)
FROM alpine:latest
RUN apk add --no-cache ca-certificates-bundle libogg libpng fftw \
    && apk del apk-tools \
    && rm -rf /var/cache/apk /etc/apk /lib/apk /usr/share/apk \
    && rm -rf /usr/share/man /usr/share/doc
RUN addgroup -g 65532 -S nonroot \
    && adduser -u 65532 -S -G nonroot -H -s /sbin/nologin nonroot
COPY --from=flac /artifacts/usr/bin/flac /usr/bin/flac
COPY --from=flac /artifacts/usr/lib/libFLAC.so* /usr/lib/
COPY --from=lame /artifacts/usr/bin/lame /usr/bin/lame
COPY --from=lame /artifacts/usr/lib/libmp3lame.so* /usr/lib/
COPY --from=sox /artifacts/usr/bin/sox_ng /usr/bin/sox_ng
COPY --from=sox /artifacts/usr/lib/libsox_ng.so* /usr/lib/
COPY --from=builder /app/target/release/caesura /bin/caesura
ENV CAESURA_DOCKER=1
USER nonroot
WORKDIR /
ENTRYPOINT ["caesura"]

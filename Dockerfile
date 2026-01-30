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

# Download imdl binary
FROM alpine:latest AS imdl
ARG TARGETARCH
ARG IMDL_VERSION=0.1.15
RUN apk add --no-cache curl
RUN case "${TARGETARCH}" in \
      amd64) ARCH="x86_64" ;; \
      arm64) ARCH="aarch64" ;; \
      *) echo "Unsupported architecture: ${TARGETARCH}" && exit 1 ;; \
    esac && \
    curl "https://github.com/casey/intermodal/releases/download/v${IMDL_VERSION}/imdl-v${IMDL_VERSION}-${ARCH}-unknown-linux-musl.tar.gz" \
      --location \
      --show-error \
      --silent \
      --connect-timeout 2 \
      --max-time 30 \
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
COPY Cargo.toml Cargo.lock /app/
COPY crates/core/Cargo.toml /app/crates/core/
COPY crates/core/build.rs /app/crates/core/
COPY crates/macros/Cargo.toml /app/crates/macros/
RUN mkdir -p crates/core/src crates/macros/src \
    && echo 'pub fn stub() {}' > crates/core/src/lib.rs \
    && echo 'fn main() {}' > crates/core/src/main.rs \
    && printf 'use proc_macro::TokenStream;\n#[proc_macro_derive(Options, attributes(options, arg, serde))]\npub fn derive_options(_: TokenStream) -> TokenStream { TokenStream::new() }\n' > crates/macros/src/lib.rs
RUN cargo fetch
RUN cargo build --release --locked \
    && rm -rf target/release/.fingerprint/caesura* target/release/deps/caesura* target/release/deps/libcaesura*
# Set the version
COPY . /app
ARG VERSION=0.0.0
RUN cargo set-version -p caesura $VERSION
# Build the release binary
RUN cargo build --release

# Build final image with minimal dependencies
FROM alpine:latest
RUN apk add --no-cache libogg lame sox imagemagick imagemagick-jpeg eyed3
COPY --from=flac-builder /flac-install/usr/bin/flac /usr/bin/flac
COPY --from=flac-builder /flac-install/usr/bin/metaflac /usr/bin/metaflac
COPY --from=flac-builder /flac-install/usr/lib/libFLAC.so* /usr/lib/
COPY --from=imdl /bin/imdl /bin/imdl
COPY --from=builder /app/target/release/caesura /bin/caesura
WORKDIR /
ENTRYPOINT ["caesura"]

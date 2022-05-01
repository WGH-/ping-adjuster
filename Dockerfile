ARG IMAGE=debian:bullseye

FROM $IMAGE as build

SHELL ["/bin/bash", "-c"]

# install Rust and cargo-deb
RUN set -eux; \
    export DEBIAN_FRONTEND=noninteractive; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        curl ca-certificates gcc libc6-dev \
# cargo-deb can use dpkg-shlibdeps to autogenerate dependencies
        dpkg-dev; \
    rm -rf /var/lib/apt/lists/*; \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s - --default-toolchain 1.60 -y; \
    source $HOME/.cargo/env; \
    rustc --version; \
    cargo --version; \
    cargo install --version 1.37.0 cargo-deb;

# build crate
ADD . /usr/src/ping-fuckuper/
# XXX https://github.com/rust-lang/cargo/issues/7124
RUN set -eux; \
    source $HOME/.cargo/env; \
    cd /usr/src/ping-fuckuper; \
    cargo build --release; \
    cargo deb; \
    mkdir /output; \
    cp target/debian/ping-fuckuper_*.deb target/release/ping-fuckuper /output; \
    cargo clean

FROM scratch
COPY --from=build /output/* .

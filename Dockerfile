FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive
WORKDIR /root

# Install General toolchain
RUN apt-get update && apt-get install -y curl git python3 perl build-essential wget
RUN apt-get install -y \
    autoconf automake autotools-dev libmpc-dev libmpfr-dev libgmp-dev \
    gawk build-essential bison flex texinfo gperf libtool patchutils bc \
    zlib1g-dev libexpat-dev ninja-build pkg-config libglib2.0-dev \
    libpixman-1-dev libsdl2-dev gcc-riscv64-unknown-elf \
    gdb-multiarch binutils-riscv64-unknown-elf

# Install qemu
ARG QEMU_VERSION=7.0.0

RUN wget https://download.qemu.org/qemu-${QEMU_VERSION}.tar.xz && \
    tar xvJf qemu-${QEMU_VERSION}.tar.xz && \
    cd /root/qemu-${QEMU_VERSION} && \
    ./configure --target-list=riscv64-softmmu,riscv64-linux-user && \
    make -j$(nproc) && \
    make install
RUN rm -rf qemu-${QEMU_VERSION} qemu-${QEMU_VERSION}.tar.xz

# Install Rust
# - https://www.rust-lang.org/tools/install

ARG RUST_VERSION=1.68
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --no-modify-path --profile minimal --default-toolchain ${RUST_VERSION} && \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME && \
    rustup --version && \
    cargo --version

RUN cargo install cargo-binutils --vers ~0.2 && \
    rustup target add riscv64gc-unknown-none-elf

RUN rustup component add clippy rustfmt

# Make GDB easier
RUN mkdir -p ~/.config/gdb
RUN echo "add-auto-load-safe-path /" > ~/.config/gdb/gdbinit

# Use tacos as the runner
COPY tacos /usr/bin

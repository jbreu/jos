FROM debian:latest

# TODO remove everything needed for gcc as we dont use it (only ld)

SHELL ["/bin/bash", "-c"]
CMD ["/bin/bash"]

ARG BINUTILS_VERSION=2.41
ARG GCC_VERSION=13.2.0

RUN echo "deb http://security.debian.org/debian-security bullseye-security main contrib non-free" > /etc/apt/sources.list

RUN set -x \
    && apt-get update \
    && apt-get install -y curl build-essential nasm xorriso grub-pc-bin grub-common make wget libgmp3-dev libmpfr-dev libisl-dev libmpc-dev texinfo make bzip2 patch sudo qemu-system-x86

# Pull binutils and gcc source code
RUN set -x \
    && mkdir -p /usr/local/src \
    && cd /usr/local/src \
    && wget -q https://ftp.gnu.org/gnu/binutils/binutils-${BINUTILS_VERSION}.tar.gz \
    && tar zxf binutils-${BINUTILS_VERSION}.tar.gz \
    && rm binutils-${BINUTILS_VERSION}.tar.gz \
    && chown -R root:root binutils-${BINUTILS_VERSION} \
    && chmod -R o-w,g+w binutils-${BINUTILS_VERSION}

# Copy compile scripts
COPY files/src /usr/local/src/

# Build and install binutils and the cross-compiler
RUN set -x \
    && cd /usr/local/src \
    && sh ./build-binutils.sh ${BINUTILS_VERSION}

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --target x86_64-unknown-none --profile complete -y

RUN source "$HOME/.cargo/env" \
    rustup target add x86_64-unknown-none

VOLUME /root/env
WORKDIR /root/env
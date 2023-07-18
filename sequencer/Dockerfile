FROM debian:bullseye-slim as builder

# Install dependencies
RUN apt update -y
RUN apt install -y curl \
                   gcc \
                   libclang-dev \
                   g++

# Install rust nightly-2023-06-19 (1.70)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain=nightly-2023-06-19
ENV PATH="/root/.cargo/bin:${PATH}"

RUN apt install -y gnupg wget

# Install LLVM 16
RUN echo "deb http://apt.llvm.org/bullseye/ llvm-toolchain-bullseye-16 main" > /etc/apt/sources.list.d/llvm-16.list
RUN echo "deb-src http://apt.llvm.org/bullseye/ llvm-toolchain-bullseye-16 main" >> /etc/apt/sources.list.d/llvm-16.list
RUN wget -O - https://apt.llvm.org/llvm-snapshot.gpg.key | apt-key add -
RUN apt update -y
RUN apt install -y llvm-16 \
                   libmlir-16-dev \
                   mlir-16-tools \
                   libpolly-16-dev

ENV MLIR_SYS_160_PREFIX=/usr/lib/llvm-16

COPY . /sequencer/

# Compile sequencer
WORKDIR /sequencer/
RUN cargo +nightly-2023-06-19 build --features benchmark --release
RUN cargo +nightly-2023-06-19 build --bin client --features benchmark --release

FROM debian:bullseye-slim as sequencer

# Copy built binaries and libs
COPY --from=builder /sequencer/target/release /sequencer/
# Copy cairo_programs
COPY --from=builder /sequencer/cairo_programs /cairo_programs

WORKDIR /

RUN apt update -y && \
    apt install -y git

# Download cairo corelib
RUN git clone --depth 1 \
              --branch v2.0.1 \
              https://github.com/starkware-libs/cairo.git \
              starkware-cairo
RUN cp -r starkware-cairo/corelib .
RUN rm -rf starkware-cairo/

CMD /sequencer/node

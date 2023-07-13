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

COPY . /sequencer/

# Compile sequencer
WORKDIR /sequencer/
RUN cargo +nightly-2023-06-19 build --release

FROM debian:bullseye-slim as sequencer

COPY --from=builder /sequencer/target/release /sequencer/

CMD /sequencer/node

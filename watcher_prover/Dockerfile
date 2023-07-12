FROM elixir:1.14.5-otp-25

# Install cargo
RUN apt-get -qq update
RUN apt-get install -y -q build-essential curl
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

# Environment variables
ENV PATH="/root/.cargo/bin:${PATH}"
ENV MIX_ENV=prod

# Copy watcher_prover inside docker
WORKDIR /watcher
COPY . /watcher

# Compile watcher_prover
RUN mix local.hex --force
RUN mix local.rebar --force
RUN mix deps.get --only $MIX_ENV
RUN mix deps.compile
RUN mix compile
RUN mix release
RUN mix phx.gen.release

CMD ./_build/prod/rel/watcher_prover/bin/watcher_prover start

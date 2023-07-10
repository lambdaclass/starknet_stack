defmodule WatcherProver.Repo do
  use Ecto.Repo,
    otp_app: :watcher_prover,
    adapter: Ecto.Adapters.Postgres
end

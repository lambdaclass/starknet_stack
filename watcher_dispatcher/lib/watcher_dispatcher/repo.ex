defmodule WatcherDispatcher.Repo do
  use Ecto.Repo,
    otp_app: :watcher_dispatcher,
    adapter: Ecto.Adapters.Postgres
end

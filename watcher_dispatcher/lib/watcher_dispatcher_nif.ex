defmodule WatcherDispatcher.NIF do
  use Rustler, otp_app: :watcher_dispatcher, crate: "watcher_dispatcher"
  def add(_a, _b), do: :erlang.nif_error(:nif_not_loaded)
  def run_program_and_get_proof(_file_name), do: :erlang.nif_error(:nif_not_loaded)
end

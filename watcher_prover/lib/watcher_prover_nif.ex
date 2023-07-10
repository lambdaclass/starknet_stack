defmodule WatcherProver.NIF do
  use Rustler, otp_app: :watcher_prover, crate: "watcher_prover"
  def add(_a, _b), do: :erlang.nif_error(:nif_not_loaded)
  def run_program_and_get_proof(_file_name), do: :erlang.nif_error(:nif_not_loaded)
end

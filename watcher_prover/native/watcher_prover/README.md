# NIF for Elixir.WatcherProverNIF

## To build the NIF module

- Your NIF will now build along with your project.

## To load the NIF module

```elixir
defmodule WatcherProverNIF do
  use Rustler, otp_app: :watcher_prover, crate: "watcherprovernif"

  # When your NIF is loaded, it will override this function.
  def add(_a, _b), do: :erlang.nif_error(:nif_not_loaded)
end
```

## Examples

[This](https://github.com/rusterlium/NifIo) is a complete example of a NIF written in Rust.

## Important Note

The prover and verifier must agree on proof options.
By the moment, we are using default proof options meant for testing.
This SHOULD be changed in the future for production.

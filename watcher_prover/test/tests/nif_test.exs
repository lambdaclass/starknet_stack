defmodule WatcherProver.NIFTest do
  use ExUnit.Case, async: true

  test "Test nif opening file" do
    file_content = File.read!("./programs/fibonacci_cairo1.casm")
    assert is_list(WatcherProver.NIF.run_program_and_get_proof(file_content))
  end
end

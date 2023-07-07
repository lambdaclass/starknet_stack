defmodule WatcherProver.NIFTest do
  use ExUnit.Case, async: true

  test "Test nif opening file" do
    file_content = File.read!("./programs/fibonacci_cairo1.casm")
    {proof, public_inputs} = WatcherProver.NIF.run_program_and_get_proof(file_content)
    assert proof != nil
    assert public_inputs != nil
  end
end

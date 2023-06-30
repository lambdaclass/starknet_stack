defmodule WatcherDispatcher.NIFTest do
  setup do
    end
  test "Test nif opening file" do
    file_content = File.read!("./programs/fiboancci_cairo1.casm")
    assert is_list(WatcherDispatcher.NIF.run_program_and_get_proof(file_content))
  end
end

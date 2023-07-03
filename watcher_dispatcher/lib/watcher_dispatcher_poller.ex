defmodule WatcherDispatcher.Poller do
  use GenServer
  use Tesla
  alias WatcherDispatcher.NIF
  plug(Tesla.Middleware.BaseUrl, "http://localhost")
  plug(Tesla.Middleware.JSON)

  require Logger

  @polling_frequency_ms 10_000

  def start_link(args) do
    GenServer.start_link(__MODULE__, args)
  end

  @impl true
  def init(_opts) do
    state = %{
      unconfirmed_blocks: [],
      highest_block: 0,
      proof_results: []
    }

    Process.send_after(self(), :poll, @polling_frequency_ms)

    {:ok, state}
  end

  def fetch_height() do
    {:ok, %{body: %{"value" => highest_block}}} = post("/starknet", %{"method" => "blockNumber"})
    {:ok, highest_block}
  end

  defp fetch_block(height) do
    {:ok, %{body: %{"value" => [block]}}} =
      post("/starknet", %{method: "getBlocksFromHeight", params: %{n: 1, height: height}})

    {:ok, block}
  end

  @impl true
  def handle_info(
        :poll,
        state = %{unconfirmed_blocks: blocks, highest_block: curr_height, proof_results: results}
      ) do
    # {:ok, fetched_height} = fetch_height()

    # # Update known latest height if
    # # it has increased, and then fetch a block.
    # state =
    #   cond do
    #     curr_height < fetched_height ->
    #       new_block = fetch_block(fetched_height)
    #       %{state | highest_block: fetched_height, unconfirmed_blocks: [new_block | blocks]}

    #     true ->
    #       state
    #   end

    # # Group blocks based on their height,
    # # we'll consider 'confirmed' as 10 blocks into the chain.
    # %{unconfirmed: unconfirmed_blocks, confirmed: confirmed_blocks} =
    #   state.unconfirmed_blocks
    #   |> Enum.group_by(fn %{"block_number" => num} ->
    #     cond do
    #       num + 10 < state.curr_height -> :confirmed
    #       true -> :unconfirmed
    #     end
    #   end)

    # # For each block, run the proof.
    # results =
    #   for block <- confirmed_blocks, into: results do
    #     run_proofs(block)
    #   end

    Logger.info("Running proof for block...")

    {:ok, program} = File.read("./programs/fibonacci_cairo1.casm")

    proof = run_proofs(program)

    Logger.info("Generated block proof #{inspect(proof)}")

    # Upload proof to S3

    Process.send_after(self(), :poll, @polling_frequency_ms)

    {:noreply, %{state | unconfirmed_blocks: [], proof_results: proof}}
  end

  # TODO: Complete this
  # For now, just call the dummy function
  def run_proofs(block) do
    NIF.run_program_and_get_proof(block)
  end
end

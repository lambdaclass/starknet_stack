defmodule WatcherProver.Poller do
  use GenServer
  use Tesla
  alias WatcherProver.NIF
  plug(Tesla.Middleware.BaseUrl, "http://localhost:5000")
  plug(Tesla.Middleware.JSON)
  alias WatcherProver.Rpc

  require Logger
  alias WatcherProver.S3

  @polling_frequency_ms 10_000
  @number_of_blocks_for_confirmation 0

  def start_link(args) do
    GenServer.start_link(__MODULE__, args)
  end

  @impl true
  def init(_opts) do
    state = %{
      first_unconfirmed_block_number: 1,
      highest_block: 0
    }

    Process.send_after(self(), :poll, @polling_frequency_ms)
    :ok = File.mkdir_p("./proofs")

    {:ok, state}
  end

  @doc """
  This handler will first poll the chain for the latest block number, check which blocks are confirmed but have not
  been proved yet, then run a proof for them and upload it to S3.
  """
  @impl true
  def handle_info(
        :poll,
        state = %{
          first_unconfirmed_block_number: first_unconfirmed_block_number,
          highest_block: highest_block
        }
      ) do
    {:ok, latest_block_number} = Rpc.last_block_number()

    updated_highest_block =
      if latest_block_number > highest_block do
        latest_block_number
      else
        highest_block
      end

    if first_unconfirmed_block_number - @number_of_blocks_for_confirmation >= highest_block do
      {:ok, block} = Rpc.get_block_by_number(first_unconfirmed_block_number)

      Logger.info(
        "Running proof for block #{block["block_hash"]} with contents #{inspect(block)}"
      )

      # TODO: fetch executions from the invoke transactions for this block to prove
      {:ok, program} = File.read("./programs/fibonacci_cairo1.casm")

      {proof, public_inputs} = run_proofs(program)

      Logger.info("Generated block proof #{inspect(proof)}")

      block_hash = block["block_hash"]
      prover_storage = Application.get_env(:watcher_prover, :prover_storage)

      case prover_storage do
        "s3" ->
          # TODO: Uncomment this when we are ready
          # :ok = S3.upload_object!(:erlang.list_to_binary(proof), block["block_hash"])
          Logger.info("Uploaded proof of block with id #{block_hash}")

        _ ->
          file_path = "./proofs/#{block_hash}.proof"
          :ok = File.write(file_path, proof, [:write])
          Logger.info("Saved block with id #{block_hash} to file ./proofs/#{block_hash}.proof")
      end

      {:noreply,
       %{
         state
         | highest_block: updated_highest_block,
           first_unconfirmed_block_number: first_unconfirmed_block_number + 1
       }}
    else
      {:noreply,
       %{
         state
         | highest_block: updated_highest_block,
           first_unconfirmed_block_number: first_unconfirmed_block_number
       }}
    end
  end

  def run_proofs(block) do
    NIF.run_program_and_get_proof(block)
  end
end

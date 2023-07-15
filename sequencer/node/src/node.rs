use crate::config::Export as _;
use crate::config::{Committee, ConfigError, Parameters, Secret};
use cairo_felt::Felt252;
use cairo_lang_compiler::CompilerConfig;
use cairo_lang_sierra::extensions::core::{CoreLibfunc, CoreType};
use cairo_lang_starknet::casm_contract_class::CasmContractClass;
use cairo_native::easy::compile_and_execute;
use cairo_vm::hint_processor::cairo_1_hint_processor::hint_processor::Cairo1HintProcessor;
use cairo_vm::serde::deserialize_program::BuiltinName;
use cairo_vm::types::program::Program;
use cairo_vm::types::relocatable::MaybeRelocatable;
use cairo_vm::vm::runners::cairo_runner::{CairoArg, CairoRunner, RunResources};
use cairo_vm::vm::vm_core::VirtualMachine;
use consensus::{Block, Consensus};
use crypto::SignatureService;
use log::info;
use mempool::{Mempool, MempoolMessage};
use num_bigint::BigUint;
use rpc_endpoint::new_server;
use rpc_endpoint::rpc::{self, InvokeTransaction, Transaction};
use sequencer::store::StoreEngine;
use serde_json::json;
use std::convert::TryInto;
use std::path::Path;
use std::sync::Arc;
use store::Store;
use tokio::sync::mpsc::{channel, Receiver};

/// The default channel capacity for this module.
pub const CHANNEL_CAPACITY: usize = 1_000;

/// Default port offset for RPC endpoint
const RPC_PORT_OFFSET: u16 = 1000;

// What type is V1(InvokeTransactionV1)?

pub struct Node {
    pub commit: Receiver<Block>,
    pub store: Store,
    pub external_store: sequencer::store::Store,
    pub sierra_program: Arc<cairo_lang_sierra::program::Program>,
}

impl Node {
    pub async fn new(
        committee_file: &str,
        key_file: &str,
        store_path: &str,
        parameters: Option<String>,
    ) -> Result<Self, ConfigError> {
        let (tx_commit, rx_commit) = channel(CHANNEL_CAPACITY);
        let (tx_consensus_to_mempool, rx_consensus_to_mempool) = channel(CHANNEL_CAPACITY);
        let (tx_mempool_to_consensus, rx_mempool_to_consensus) = channel(CHANNEL_CAPACITY);

        // Read the committee and secret key from file.
        let committee = Committee::read(committee_file)?;
        let secret = Secret::read(key_file)?;
        let name = secret.name;
        let secret_key = secret.secret;

        // Load default parameters if none are specified.
        let parameters = match parameters {
            Some(filename) => Parameters::read(&filename)?,
            None => Parameters::default(),
        };

        // Make the data store.
        let store = Store::new(store_path).expect("Failed to create store");
        let external_store =
            sequencer::store::Store::new(store_path, sequencer::store::EngineType::Sled);

        // Compile fibonacci to Sierra
        let sierra_program: Arc<cairo_lang_sierra::program::Program> =
            cairo_lang_compiler::compile_cairo_project_at_path(
                Path::new("../cairo_programs/fib_contract.cairo"),
                CompilerConfig {
                    replace_ids: true,
                    ..Default::default()
                },
            )
            .unwrap();

        // Run the signature service.
        let signature_service = SignatureService::new(secret_key);

        // Make a new mempool.
        Mempool::spawn(
            name,
            committee.clone().mempool,
            parameters.mempool,
            store.clone(),
            rx_consensus_to_mempool,
            tx_mempool_to_consensus,
        );

        // Run the consensus core.
        Consensus::spawn(
            name,
            committee.clone().consensus,
            parameters.consensus,
            signature_service,
            store.clone(),
            rx_mempool_to_consensus,
            tx_consensus_to_mempool,
            tx_commit,
        );

        let external_store_clone = external_store.clone();
        tokio::spawn(async move {
            let port = committee
                .mempool
                .mempool_address(&name)
                .expect("Our public key is not in the committee")
                .port()
                + RPC_PORT_OFFSET;

            let handle = new_server(port, external_store_clone).await;

            match handle {
                Ok(handle) => {
                    info!("RPC Server started, running on port {}", port);
                    handle.stopped().await;
                }
                Err(e) => println!("Error creating RPC server: {}", e),
            };
        });

        info!("Node {} successfully booted", name);
        Ok(Self {
            commit: rx_commit,
            store,
            external_store,
            sierra_program: sierra_program,
        })
    }

    pub fn print_key_file(filename: &str) -> Result<(), ConfigError> {
        Secret::new().write(filename)
    }

    pub async fn analyze_block(&mut self) {
        while let Some(block) = self.commit.recv().await {
            // This is where we can further process committed block.
            for p in block.payload {
                let tx_batch = self.store.read(p.to_vec()).await.unwrap().unwrap();
                info!("Batch is {} bytes long", tx_batch.len());

                let list_of_tx: MempoolMessage =
                    bincode::deserialize(&tx_batch).expect("Error trying to deserialize batch");
                //info!("There are {} transactions in {:?} ", list_of_tx.len(), p);

                match list_of_tx {
                    MempoolMessage::Batch(batch_txs) => {
                        info!(
                            "Batch message confirmed, with {} transactions!",
                            batch_txs.len()
                        );

                        let mut transactions = vec![];
                        for (i, tx_bytes) in batch_txs.into_iter().enumerate() {
                            // Consensus codebase uses the first 9 bytes to track the transaction like this:
                            //
                            // - First byte can be 0 or 1 and represents whether it's a benchmarked tx or standard tx
                            // - Next 8 bytes represent a transaction ID
                            //
                            // If it's a benchmarked tx, it then gets tracked in logs to compute metrics
                            // So we need to strip that section in order to get the starknet transaction to execute
                            #[cfg(feature = "benchmark")]
                            let tx_bytes = &tx_bytes[9..];

                            let starknet_tx = rpc::Transaction::from_bytes(&tx_bytes);
                            info!("Message {i} in {:?} is of tx_type {:?}", p, starknet_tx);
                            let n = 10_usize;

                            // TODO create a execution engine structure to improve code quality
                            // In this case we are executing cairo_native
                            let is_cairo_vm = false;
                            if is_cairo_vm {
                                let program =
                                    include_bytes!("../../cairo_programs/fib_contract.casm");
                                let ret = run_cairo_1_entrypoint(
                                    program.as_slice(),
                                    0,
                                    &[0_usize.into(), 1_usize.into(), n.into()],
                                );
                                info!("Output: ret is {:?}", ret);
                            } else {
                                let a = get_input_value_cairo_native(0_u32);
                                let b = get_input_value_cairo_native(1_u32);
                                let n = get_input_value_cairo_native(n as u32);
                                let ret =
                                    execute_fibonacci_cairo_native(&self.sierra_program, a, b, n);
                                info!("Output: ret is {:?}", ret);
                            }

                            let starknet_tx_string = serde_json::to_string(&starknet_tx).unwrap();

                            match &starknet_tx {
                                Transaction::Invoke(InvokeTransaction::V1(tx)) => {
                                    info!(
                                        "tx hash serialized: {}, decimal {} (hex {})",
                                        serde_json::to_string(&tx.transaction_hash).unwrap(),
                                        &tx.transaction_hash,
                                        &tx.transaction_hash.to_str_radix(16)
                                    );

                                    let _ = self.external_store.add_transaction(
                                        tx.transaction_hash.to_be_bytes().to_vec(),
                                        starknet_tx_string.into_bytes(),
                                    );
                                }
                                _ => todo!(),
                            }

                            transactions.push(starknet_tx);
                        }

                        // TODO create a correct Block Structure instad of a hardcoded one
                        let block = rpc::BlockWithTxs {
                            status: rpc_endpoint::rpc::BlockStatus::AcceptedOnL2,
                            block_hash: Felt252::new(11239218),
                            parent_hash: Felt252::new(19203123),
                            block_number: 1,
                            new_root: Felt252::new(938938281),
                            timestamp: 1688498274,
                            sequencer_address: Felt252::new(12039102),
                            transactions,
                        };
                        let block_id = block.block_number;
                        let block_serialized: Vec<u8> =
                            serde_json::to_string(&rpc::MaybePendingBlockWithTxs::Block(block))
                                .unwrap()
                                .as_bytes()
                                .to_vec();

                        let _ = self
                            .external_store
                            .add_block(block_id.to_be_bytes().to_vec(), block_serialized);
                    }
                    MempoolMessage::BatchRequest(_, _) => {
                        info!("Batch Request message confirmed")
                    }
                }
            }
        }
    }
}

// TODO: Move this to a separate library file
fn run_cairo_1_entrypoint(
    program_content: &[u8],
    entrypoint_offset: usize,
    args: &[MaybeRelocatable],
) -> Vec<cairo_vm::felt::Felt252> {
    let contract_class: CasmContractClass = serde_json::from_slice(program_content).unwrap();
    let mut hint_processor =
        Cairo1HintProcessor::new(&contract_class.hints, RunResources::default());
    let aux_program: Program = contract_class.clone().try_into().unwrap();
    let mut runner = CairoRunner::new(
        &(contract_class.clone().try_into().unwrap()),
        "all_cairo",
        false,
    )
    .unwrap();
    let mut vm = VirtualMachine::new(false);

    let program_builtins = get_casm_contract_builtins(&contract_class, entrypoint_offset);
    runner
        .initialize_function_runner_cairo_1(&mut vm, &program_builtins)
        .unwrap();

    // Implicit Args
    let syscall_segment = MaybeRelocatable::from(vm.add_memory_segment());

    let builtins: Vec<&'static str> = runner
        .get_program_builtins()
        .iter()
        .map(|b| b.name())
        .collect();

    let builtin_segment: Vec<MaybeRelocatable> = vm
        .get_builtin_runners()
        .iter()
        .filter(|b| builtins.contains(&b.name()))
        .flat_map(|b| b.initial_stack())
        .collect();

    let initial_gas = MaybeRelocatable::from(usize::MAX);

    let mut implicit_args = builtin_segment;
    implicit_args.extend([initial_gas]);
    implicit_args.extend([syscall_segment]);

    // Other args

    // Load builtin costs
    let builtin_costs: Vec<MaybeRelocatable> =
        vec![0.into(), 0.into(), 0.into(), 0.into(), 0.into()];
    let builtin_costs_ptr = vm.add_memory_segment();
    vm.load_data(builtin_costs_ptr, &builtin_costs).unwrap();

    // Load extra data
    let core_program_end_ptr = (runner.program_base.unwrap() + aux_program.data_len()).unwrap();
    let program_extra_data: Vec<MaybeRelocatable> =
        vec![0x208B7FFF7FFF7FFE.into(), builtin_costs_ptr.into()];
    vm.load_data(core_program_end_ptr, &program_extra_data)
        .unwrap();

    // Load calldata
    let calldata_start = vm.add_memory_segment();
    let calldata_end = vm.load_data(calldata_start, &args.to_vec()).unwrap();

    // Create entrypoint_args

    let mut entrypoint_args: Vec<CairoArg> = implicit_args
        .iter()
        .map(|m| CairoArg::from(m.clone()))
        .collect();
    entrypoint_args.extend([
        MaybeRelocatable::from(calldata_start).into(),
        MaybeRelocatable::from(calldata_end).into(),
    ]);
    let entrypoint_args: Vec<&CairoArg> = entrypoint_args.iter().collect();

    // Run contract entrypoint

    runner
        .run_from_entrypoint(
            entrypoint_offset,
            &entrypoint_args,
            true,
            Some(aux_program.data_len() + program_extra_data.len()),
            &mut vm,
            &mut hint_processor,
        )
        .unwrap();

    // Check return values
    let return_values = vm.get_return_values(5).unwrap();
    let retdata_start = return_values[3].get_relocatable().unwrap();
    let retdata_end = return_values[4].get_relocatable().unwrap();
    let retdata: Vec<cairo_vm::felt::Felt252> = vm
        .get_integer_range(retdata_start, (retdata_end - retdata_start).unwrap())
        .unwrap()
        .iter()
        .map(|c| c.clone().into_owned())
        .collect();
    retdata
}

fn get_casm_contract_builtins(
    contract_class: &CasmContractClass,
    entrypoint_offset: usize,
) -> Vec<BuiltinName> {
    contract_class
        .entry_points_by_type
        .external
        .iter()
        .find(|e| e.offset == entrypoint_offset)
        .unwrap()
        .builtins
        .iter()
        .map(|n| format!("{}_builtin", n))
        .map(|s| match &*s {
            cairo_vm::vm::runners::builtin_runner::OUTPUT_BUILTIN_NAME => BuiltinName::output,
            cairo_vm::vm::runners::builtin_runner::RANGE_CHECK_BUILTIN_NAME => {
                BuiltinName::range_check
            }
            cairo_vm::vm::runners::builtin_runner::HASH_BUILTIN_NAME => BuiltinName::pedersen,
            cairo_vm::vm::runners::builtin_runner::SIGNATURE_BUILTIN_NAME => BuiltinName::ecdsa,
            cairo_vm::vm::runners::builtin_runner::KECCAK_BUILTIN_NAME => BuiltinName::keccak,
            cairo_vm::vm::runners::builtin_runner::BITWISE_BUILTIN_NAME => BuiltinName::bitwise,
            cairo_vm::vm::runners::builtin_runner::EC_OP_BUILTIN_NAME => BuiltinName::ec_op,
            cairo_vm::vm::runners::builtin_runner::POSEIDON_BUILTIN_NAME => BuiltinName::poseidon,
            cairo_vm::vm::runners::builtin_runner::SEGMENT_ARENA_BUILTIN_NAME => {
                BuiltinName::segment_arena
            }
            _ => panic!("Invalid builtin {}", s),
        })
        .collect()
}

fn get_input_value_cairo_native(n: u32) -> Vec<u32> {
    let mut digits = BigUint::from(n).to_u32_digits();
    digits.resize(8, 0);
    digits
}

fn execute_fibonacci_cairo_native(
    sierra_program: &Arc<cairo_lang_sierra::program::Program>,
    a: Vec<u32>,
    b: Vec<u32>,
    n: Vec<u32>,
) -> u64 {
    std::env::set_var(
        "CARGO_MANIFEST_DIR",
        format!("{}/a", std::env::var("CARGO_MANIFEST_DIR").unwrap()),
    );

    let program = sierra_program;
    let mut writer: Vec<u8> = Vec::new();
    let mut res = serde_json::Serializer::new(&mut writer);
    compile_and_execute::<CoreType, CoreLibfunc, _, _>(
        &program,
        &program
            .funcs
            .iter()
            .find(|x| {
                x.id.debug_name.as_deref() == Some("fib_contract::fib_contract::Fibonacci::fib")
            })
            .unwrap()
            .id,
        json!([null, 9000, a, b, n]),
        &mut res,
    )
    .unwrap();

    // The output expected as a string will be a json that looks like this:
    // [null,9000,[0,[[55,0,0,0,0,0,0,0]]]]
    let deserialized_result: String = String::from_utf8(writer).unwrap();
    let deserialized_value = serde_json::from_str::<serde_json::Value>(&deserialized_result)
        .expect("Failed to deserialize result");
    deserialized_value[2][1][0][0].as_u64().unwrap()
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use cairo_lang_compiler::CompilerConfig;

    #[test]
    fn fib_1_cairovm() {
        let program = include_bytes!("../../cairo_programs/fib_contract.casm");
        let n = 1_usize;
        let ret = super::run_cairo_1_entrypoint(
            program.as_slice(),
            0,
            &[1_usize.into(), 1_usize.into(), n.into()],
        );
        assert_eq!(ret, vec![1_usize.into()]);
    }

    #[test]
    fn fib_10_cairovm() {
        let program = include_bytes!("../../cairo_programs/fib_contract.casm");
        let n = 10_usize;
        let ret = super::run_cairo_1_entrypoint(
            program.as_slice(),
            0,
            &[0_usize.into(), 1_usize.into(), n.into()],
        );
        assert_eq!(ret, vec![55_usize.into()]);
    }

    #[test]
    fn fib_10_cairo_native() {
        let a = super::get_input_value_cairo_native(0_u32);

        let b = super::get_input_value_cairo_native(1_u32);

        let n = super::get_input_value_cairo_native(10_u32);

        let sierra_program = cairo_lang_compiler::compile_cairo_project_at_path(
            Path::new("../cairo_programs/fib_contract.cairo"),
            CompilerConfig {
                replace_ids: true,
                ..Default::default()
            },
        )
        .unwrap();

        let fib_10 = super::execute_fibonacci_cairo_native(&sierra_program, a, b, n);
        assert_eq!(fib_10, 55);
    }

    #[test]
    fn get_input_value_cairo_native_should_be_10() {
        let input = super::get_input_value_cairo_native(10);
        assert_eq!(input, vec![10, 0, 0, 0, 0, 0, 0, 0]);
    }
}

use cairo_felt::Felt252;
use cairo_lang_sierra::extensions::core::{CoreLibfunc, CoreType};
use cairo_lang_sierra::program::Program as SierraProgram;
use cairo_lang_sierra::ProgramParser;
use cairo_native::easy::compile_and_execute;
use cairo_native::starknet::{
    BlockInfo, ExecutionInfo, StarkNetSyscallHandler, SyscallResult, TxInfo, U256,
};
use num_bigint::BigUint;
use serde_json::json;
use std::sync::Arc;

pub struct CairoNativeEngine {
    fib_program: Arc<SierraProgram>,
    fact_program: Arc<SierraProgram>,
}

impl CairoNativeEngine {
    pub fn new(fib_program: Arc<SierraProgram>, fact_program: Arc<SierraProgram>) -> Self {
        Self {
            fib_program,
            fact_program,
        }
    }

    pub fn execute_fibonacci(&self, n: Vec<u32>) -> String {
        let ret: u64 = execute_cairo_native_program(
            &self.fib_program,
            "fib_contract::fib_contract::Fibonacci::fib",
            vec![
                get_input_value_cairo_native(0),
                get_input_value_cairo_native(1),
                n,
            ],
        );
        format!("Output Fib Cairo Native: {:?}", ret)
    }

    pub fn execute_factorial(&self, n: Vec<u32>) -> String {
        let ret: u64 = execute_cairo_native_program(
            &self.fact_program,
            "fact_contract::fact_contract::Factorial::fact",
            vec![n],
        );
        format!("Output Fact Cairo Native: {:?}", ret)
    }
}

fn get_input_value_cairo_native(n: usize) -> Vec<u32> {
    let mut digits = BigUint::from(n).to_u32_digits();
    digits.resize(8, 0);
    digits
}

fn execute_cairo_native_program(
    sierra_program: &Arc<SierraProgram>,
    entrypoint: &str,
    args: Vec<Vec<u32>>,
) -> u64 {
    let program = sierra_program;
    let mut writer: Vec<u8> = Vec::new();
    let mut res = serde_json::Serializer::new(&mut writer);
    // use params variable that is a deserializable variable
    let mut params = json!([null, 9000]);
    for arg in args {
        params.as_array_mut().unwrap().push(arg.into());
    }
    compile_and_execute::<CoreType, CoreLibfunc, _, _>(
        program,
        &program
            .funcs
            .iter()
            .find(|x| x.id.debug_name.as_deref() == Some(entrypoint))
            .unwrap()
            .id,
        params,
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

#[derive(Debug)]
struct SyscallHandler;

impl StarkNetSyscallHandler for SyscallHandler {
    fn get_block_hash(&self, block_number: u64) -> SyscallResult<cairo_felt::Felt252> {
        println!("Called `get_block_hash({block_number})` from MLIR.");
        Ok(Felt252::from_bytes_be(b"get_block_hash ok"))
    }

    fn get_execution_info(&self) -> SyscallResult<cairo_native::starknet::ExecutionInfo> {
        println!("Called `get_execution_info()` from MLIR.");
        Ok(ExecutionInfo {
            block_info: BlockInfo {
                block_number: 1234,
                block_timestamp: 2345,
                sequencer_address: 3456.into(),
            },
            tx_info: TxInfo {
                version: 4567.into(),
                account_contract_address: 5678.into(),
                max_fee: 6789,
                signature: vec![1248.into(), 2486.into()],
                transaction_hash: 9876.into(),
                chain_id: 8765.into(),
                nonce: 7654.into(),
            },
            caller_address: 6543.into(),
            contract_address: 5432.into(),
            entry_point_selector: 4321.into(),
        })
    }

    fn deploy(
        &self,
        class_hash: cairo_felt::Felt252,
        contract_address_salt: cairo_felt::Felt252,
        calldata: &[cairo_felt::Felt252],
        deploy_from_zero: bool,
    ) -> SyscallResult<(cairo_felt::Felt252, Vec<cairo_felt::Felt252>)> {
        println!("Called `deploy({class_hash}, {contract_address_salt}, {calldata:?}, {deploy_from_zero})` from MLIR.");
        Ok((
            class_hash + contract_address_salt,
            calldata.iter().map(|x| x + &Felt252::new(1)).collect(),
        ))
    }

    fn replace_class(&self, class_hash: cairo_felt::Felt252) -> SyscallResult<()> {
        println!("Called `replace_class({class_hash})` from MLIR.");
        Ok(())
    }

    fn library_call(
        &self,
        class_hash: cairo_felt::Felt252,
        function_selector: cairo_felt::Felt252,
        calldata: &[cairo_felt::Felt252],
    ) -> SyscallResult<Vec<cairo_felt::Felt252>> {
        println!(
            "Called `library_call({class_hash}, {function_selector}, {calldata:?})` from MLIR."
        );
        Ok(calldata.iter().map(|x| x * &Felt252::new(3)).collect())
    }

    fn call_contract(
        &self,
        address: cairo_felt::Felt252,
        entry_point_selector: cairo_felt::Felt252,
        calldata: &[cairo_felt::Felt252],
    ) -> SyscallResult<Vec<cairo_felt::Felt252>> {
        println!(
            "Called `call_contract({address}, {entry_point_selector}, {calldata:?})` from MLIR."
        );
        Ok(calldata.iter().map(|x| x * &Felt252::new(3)).collect())
    }

    fn storage_read(
        &self,
        address_domain: u32,
        address: cairo_felt::Felt252,
    ) -> SyscallResult<cairo_felt::Felt252> {
        println!("Called `storage_read({address_domain}, {address})` from MLIR.");
        Ok(address * &Felt252::new(3))
    }

    fn storage_write(
        &self,
        address_domain: u32,
        address: cairo_felt::Felt252,
        value: cairo_felt::Felt252,
    ) -> SyscallResult<()> {
        println!("Called `storage_write({address_domain}, {address}, {value})` from MLIR.");
        Ok(())
    }

    fn emit_event(
        &self,
        keys: &[cairo_felt::Felt252],
        data: &[cairo_felt::Felt252],
    ) -> SyscallResult<()> {
        println!("Called `emit_event({keys:?}, {data:?})` from MLIR.");
        Ok(())
    }

    fn send_message_to_l1(
        &self,
        to_address: cairo_felt::Felt252,
        payload: &[cairo_felt::Felt252],
    ) -> SyscallResult<()> {
        println!("Called `send_message_to_l1({to_address}, {payload:?})` from MLIR.");
        Ok(())
    }

    fn keccak(&self, input: &[u64]) -> SyscallResult<cairo_native::starknet::U256> {
        println!("Called `keccak({input:?})` from MLIR.");
        Ok(U256(Felt252::from(1234567890).to_le_bytes()))
    }

    fn secp256k1_add(
        &self,
        _p0: cairo_native::starknet::Secp256k1Point,
        _p1: cairo_native::starknet::Secp256k1Point,
    ) -> SyscallResult<Option<cairo_native::starknet::Secp256k1Point>> {
        todo!()
    }

    fn secp256k1_get_point_from_x(
        &self,
        _x: cairo_native::starknet::U256,
        _y_parity: bool,
    ) -> SyscallResult<Option<cairo_native::starknet::Secp256k1Point>> {
        todo!()
    }

    fn secp256k1_get_xy(
        &self,
        _p: cairo_native::starknet::Secp256k1Point,
    ) -> SyscallResult<(cairo_native::starknet::U256, cairo_native::starknet::U256)> {
        todo!()
    }

    fn secp256k1_mul(
        &self,
        _p: cairo_native::starknet::Secp256k1Point,
        _m: cairo_native::starknet::U256,
    ) -> SyscallResult<Option<cairo_native::starknet::Secp256k1Point>> {
        todo!()
    }

    fn secp256k1_new(
        &self,
        _x: cairo_native::starknet::U256,
        _y: cairo_native::starknet::U256,
    ) -> SyscallResult<Option<cairo_native::starknet::Secp256k1Point>> {
        todo!()
    }

    fn secp256r1_add(
        &self,
        _p0: cairo_native::starknet::Secp256k1Point,
        _p1: cairo_native::starknet::Secp256k1Point,
    ) -> SyscallResult<Option<cairo_native::starknet::Secp256k1Point>> {
        todo!()
    }

    fn secp256r1_get_point_from_x(
        &self,
        _x: cairo_native::starknet::U256,
        _y_parity: bool,
    ) -> SyscallResult<Option<cairo_native::starknet::Secp256k1Point>> {
        todo!()
    }

    fn secp256r1_get_xy(
        &self,
        _p: cairo_native::starknet::Secp256k1Point,
    ) -> SyscallResult<(cairo_native::starknet::U256, cairo_native::starknet::U256)> {
        todo!()
    }

    fn secp256r1_mul(
        &self,
        _p: cairo_native::starknet::Secp256k1Point,
        _m: cairo_native::starknet::U256,
    ) -> SyscallResult<Option<cairo_native::starknet::Secp256k1Point>> {
        todo!()
    }

    fn secp256r1_new(
        &self,
        _x: cairo_native::starknet::U256,
        _y: cairo_native::starknet::U256,
    ) -> SyscallResult<Option<cairo_native::starknet::Secp256k1Point>> {
        todo!()
    }

    fn pop_log(&self) {
        todo!()
    }

    fn set_account_contract_address(&self, _contract_address: cairo_felt::Felt252) {
        todo!()
    }

    fn set_block_number(&self, _block_number: u64) {
        todo!()
    }

    fn set_block_timestamp(&self, _block_timestamp: u64) {
        todo!()
    }

    fn set_caller_address(&self, _address: cairo_felt::Felt252) {
        todo!()
    }

    fn set_chain_id(&self, _chain_id: cairo_felt::Felt252) {
        todo!()
    }

    fn set_contract_address(&self, _address: cairo_felt::Felt252) {
        todo!()
    }

    fn set_max_fee(&self, _max_fee: u128) {
        todo!()
    }

    fn set_nonce(&self, _nonce: cairo_felt::Felt252) {
        todo!()
    }

    fn set_sequencer_address(&self, _address: cairo_felt::Felt252) {
        todo!()
    }

    fn set_signature(&self, _signature: &[cairo_felt::Felt252]) {
        todo!()
    }

    fn set_transaction_hash(&self, _transaction_hash: cairo_felt::Felt252) {
        todo!()
    }

    fn set_version(&self, _version: cairo_felt::Felt252) {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use std::{fs, path::Path, sync::Arc, io};

    use cairo_lang_compiler::{compile_cairo_project_at_path, CompilerConfig};
    use cairo_lang_sierra::{
        extensions::core::{CoreLibfunc, CoreType},
        program::Program,
        program_registry::ProgramRegistry,
        ProgramParser,
    };
    use cairo_native::{metadata::{
        runtime_bindings::RuntimeBindingsMeta, syscall_handler::SyscallHandlerMeta, MetadataStorage,
    }, utils::register_runtime_symbols};
    use serde_json::json;

    use crate::cairo_native_engine::{execute_cairo_native_program, get_input_value_cairo_native};

    use super::SyscallHandler;
    use melior::{
        dialect::DialectRegistry,
        ir::{Location, Module},
        pass::{self, PassManager},
        utility::{register_all_dialects, register_all_passes},
        Context, ExecutionEngine,
    };

    #[test]
    fn fib_10_cairo_native() {
        let a = super::get_input_value_cairo_native(0_usize);

        let b = super::get_input_value_cairo_native(1_usize);

        let n = super::get_input_value_cairo_native(10_usize);

        let sierra_program = compile_cairo_project_at_path(
            Path::new("../cairo_programs/fib_contract.cairo"),
            CompilerConfig {
                replace_ids: true,
                ..Default::default()
            },
        )
        .unwrap();

        let fib_10 = execute_cairo_native_program(
            &sierra_program,
            "fib_contract::fib_contract::Fibonacci::fib",
            vec![a, b, n],
        );
        assert_eq!(fib_10, 55);
    }

    #[test]
    fn compile_erc20_cairo_native() {
        let program_src = fs::read_to_string("../cairo_programs/erc20.sierra").unwrap();
        let program_parser = ProgramParser::new();
        let program: Arc<Program> = Arc::new(program_parser.parse(&program_src).unwrap());
        let entry_point = match program
            .funcs
            .iter()
            .find(|x| x.id.debug_name.as_deref() == Some("erc20::erc20::erc_20::constructor"))
        {
            Some(x) => x,
            None => {
                panic!("No entry point found");
            }
        };
        // Initialize MLIR.
        let context = Context::new();
        context.append_dialect_registry(&{
            let registry = DialectRegistry::new();
            register_all_dialects(&registry);
            registry
        });
        context.load_all_available_dialects();

        register_all_passes();

        // Compile the program.
        let mut module = Module::new(Location::unknown(&context));
        let mut metadata = MetadataStorage::new();
        let registry = ProgramRegistry::<CoreType, CoreLibfunc>::new(&program).unwrap();

        // Make the runtime library available.
        metadata.insert(RuntimeBindingsMeta::default()).unwrap();

        // Make the Starknet syscall handler available.
        metadata
            .insert(SyscallHandlerMeta::new(&SyscallHandler))
            .unwrap();

        cairo_native::compile::<CoreType, CoreLibfunc>(
            &context,
            &module,
            &program,
            &registry,
            &mut metadata,
            None,
        )
        .unwrap();

        // Lower to LLVM.
        let pass_manager = PassManager::new(&context);
        pass_manager.enable_verifier(true);
        pass_manager.add_pass(pass::transform::create_canonicalizer());

        pass_manager.add_pass(pass::conversion::create_scf_to_control_flow());

        pass_manager.add_pass(pass::conversion::create_arith_to_llvm());
        pass_manager.add_pass(pass::conversion::create_control_flow_to_llvm());
        pass_manager.add_pass(pass::conversion::create_func_to_llvm());
        pass_manager.add_pass(pass::conversion::create_index_to_llvm_pass());
        pass_manager.add_pass(pass::conversion::create_mem_ref_to_llvm());
        pass_manager.add_pass(pass::conversion::create_reconcile_unrealized_casts());

        pass_manager.run(&mut module).unwrap();

        // Create the JIT engine.
        // There is a segmentation fault here. It seems that the problem is that the module is not well formed.
        let engine = ExecutionEngine::new(&module, 3, &[], false);

        // #[cfg(feature = "with-runtime")]
        // register_runtime_symbols(&engine);

        // let params_input = json!([
        //     (),
        //     u64::MAX,
        //     metadata
        //         .get::<SyscallHandlerMeta>()
        //         .unwrap()
        //         .as_ptr()
        //         .addr()
        // ]);
    
        // cairo_native::execute(
        //     &engine,
        //     &registry,
        //     &entry_point.id,
        //     params_input,
        //     &mut serde_json::Serializer::pretty(io::stdout()),
        // )
        // .unwrap();
        // println!();


    }

    #[test]
    fn fact_10_cairo_native() {
        let n = super::get_input_value_cairo_native(10_usize);

        let sierra_program = compile_cairo_project_at_path(
            Path::new("../cairo_programs/fact_contract.cairo"),
            CompilerConfig {
                replace_ids: true,
                ..Default::default()
            },
        )
        .unwrap();

        let fact_10 = execute_cairo_native_program(
            &sierra_program,
            "fact_contract::fact_contract::Factorial::fact",
            vec![n],
        );
        assert_eq!(fact_10, 3628800);
    }

    #[test]
    fn get_input_value_cairo_native_should_be_10() {
        let input = get_input_value_cairo_native(10);
        assert_eq!(input, vec![10, 0, 0, 0, 0, 0, 0, 0]);
    }
}

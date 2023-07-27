use cairo_lang_starknet::casm_contract_class::CasmContractClass;
use cairo_vm::felt::Felt252;
use cairo_vm::hint_processor::cairo_1_hint_processor::hint_processor::Cairo1HintProcessor;
use cairo_vm::serde::deserialize_program::BuiltinName;
use cairo_vm::types::program::Program;
use cairo_vm::types::relocatable::MaybeRelocatable;
use cairo_vm::vm::runners::cairo_runner::CairoArg;
use cairo_vm::vm::runners::cairo_runner::{CairoRunner, RunResources};
use cairo_vm::vm::vm_core::VirtualMachine;

pub struct CairoVMEngine {
    fib_program: CasmContractClass,
    fact_program: CasmContractClass,
    fib_builtins: Vec<BuiltinName>,
    fact_builtins: Vec<BuiltinName>,
}

impl CairoVMEngine {
    pub fn new(raw_fib_program: Vec<u8>, raw_fact_program: Vec<u8>) -> Self {
        let fib_program: CasmContractClass = serde_json::from_slice(&raw_fib_program).unwrap();
        let fact_program: CasmContractClass = serde_json::from_slice(&raw_fact_program).unwrap();
        let fib_builtins = get_casm_contract_builtins(&fib_program, 0);
        let fact_builtins = get_casm_contract_builtins(&fact_program, 0);
        Self {
            fib_program,
            fact_program,
            fib_builtins,
            fact_builtins,
        }
    }

    // TODO: include this function to node execution for arbitrary code execution
    pub fn execute_program(
        &self,
        program: Vec<u8>,
        entrypoint_offset: usize,
        args: &[MaybeRelocatable],
    ) -> Vec<Felt252> {
        let contract_class: CasmContractClass = serde_json::from_slice(&program).unwrap();
        let program_builtins = get_casm_contract_builtins(&contract_class, entrypoint_offset);
        run_cairo_1_entrypoint(&contract_class, &program_builtins, entrypoint_offset, args)
    }

    pub fn execute_fibonacci(&self, n: usize) -> String {
        let res = run_cairo_1_entrypoint(
            &self.fib_program,
            &self.fib_builtins,
            0,
            &[0_usize.into(), 1_usize.into(), n.into()],
        );
        format!("Output Fib Cairo VM: {:?}", res)
    }

    pub fn execute_factorial(&self, n: usize) -> String {
        let res = run_cairo_1_entrypoint(&self.fact_program, &self.fact_builtins, 0, &[n.into()]);
        format!("Output Fact Cairo VM: {:?}", res)
    }
}

fn run_cairo_1_entrypoint(
    program_content: &CasmContractClass,
    program_builtins: &[BuiltinName],
    entrypoint_offset: usize,
    args: &[MaybeRelocatable],
) -> Vec<Felt252> {
    let contract_class = program_content;
    let mut hint_processor =
        Cairo1HintProcessor::new(&contract_class.hints, RunResources::default());
    let program: Program = contract_class.clone().try_into().unwrap();
    let mut runner = CairoRunner::new(
        &(contract_class.clone().try_into().unwrap()),
        "all_cairo",
        false,
    )
    .unwrap();
    let mut vm = VirtualMachine::new(false);

    runner
        .initialize_function_runner_cairo_1(&mut vm, program_builtins)
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

    // Load builtin costs
    let builtin_costs: Vec<MaybeRelocatable> =
        vec![0.into(), 0.into(), 0.into(), 0.into(), 0.into()];
    let builtin_costs_ptr = vm.add_memory_segment();
    vm.load_data(builtin_costs_ptr, &builtin_costs).unwrap();

    // Load extra data
    let core_program_end_ptr = (runner.program_base.unwrap() + program.data_len()).unwrap();
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
            Some(program.data_len() + program_extra_data.len()),
            &mut vm,
            &mut hint_processor,
        )
        .unwrap();

    // Check return values
    let return_values = vm.get_return_values(5).unwrap();
    let retdata_start = return_values[3].get_relocatable().unwrap();
    let retdata_end = return_values[4].get_relocatable().unwrap();
    let retdata: Vec<Felt252> = vm
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

#[cfg(test)]
mod tests {
    use cairo_lang_starknet::casm_contract_class::CasmContractClass;

    use crate::cairovm_engine::{
        get_casm_contract_builtins, run_cairo_1_entrypoint, CairoVMEngine,
    };

    #[test]
    fn fib_1_run_cairo_1_entrypoint() {
        let program_bytes = include_bytes!("../../cairo_programs/fib_contract.casm");
        let program = serde_json::from_slice::<CasmContractClass>(program_bytes).unwrap();
        let program_builtins = get_casm_contract_builtins(&program, 0);
        let n = 1_usize;
        let ret = run_cairo_1_entrypoint(
            &program,
            &program_builtins,
            0,
            &[1_usize.into(), 1_usize.into(), n.into()],
        );
        assert_eq!(ret, vec![1_usize.into()]);
    }

    #[test]
    fn fib_10_run_cairo_1_entrypoint() {
        let program_bytes = include_bytes!("../../cairo_programs/fib_contract.casm");
        let program = serde_json::from_slice::<CasmContractClass>(program_bytes).unwrap();
        let program_builtins = get_casm_contract_builtins(&program, 0);
        let n = 10_usize;
        let ret = run_cairo_1_entrypoint(
            &program,
            &program_builtins,
            0,
            &[0_usize.into(), 1_usize.into(), n.into()],
        );
        assert_eq!(ret, vec![55_usize.into()]);
    }

    #[test]
    fn create_and_execute_cairovm_engine() {
        let fib_program = include_bytes!("../../cairo_programs/fib_contract.casm");
        let fact_program = include_bytes!("../../cairo_programs/fact_contract.casm");
        let engine = CairoVMEngine::new(fib_program.to_vec(), fact_program.to_vec());
        assert_eq!(engine.execute_fibonacci(10), "Output Fib Cairo VM: [55]");
        assert_eq!(
            engine.execute_factorial(10),
            "Output Fact Cairo VM: [3628800]"
        );
    }
}

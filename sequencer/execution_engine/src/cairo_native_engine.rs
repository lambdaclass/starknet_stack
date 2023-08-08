use cairo_lang_sierra::{
    extensions::core::{CoreLibfunc, CoreType},
    program::Program,
};
use cairo_native::metadata::syscall_handler::SyscallHandlerMeta;
use num_bigint::BigUint;
use serde_json::{json, Value};
use std::sync::Arc;

pub mod syscall_handler;

pub struct CairoNativeEngine {
    fib_program: Arc<Program>,
    fact_program: Arc<Program>,
    erc20_program: Arc<Program>,
}

impl CairoNativeEngine {
    pub fn new(
        fib_program: Arc<Program>,
        fact_program: Arc<Program>,
        erc20_program: Arc<Program>,
    ) -> Self {
        Self {
            fib_program,
            fact_program,
            erc20_program,
        }
    }

    pub fn execute_fibonacci(&self, n: Vec<u32>) -> String {
        let ret = execute_cairo_native_program(
            &self.fib_program,
            "fib_contract::fib_contract::Fibonacci::fib",
            vec![
                get_input_value_cairo_native(0),
                get_input_value_cairo_native(1),
                n,
            ],
            false,
        );
        format!("Output Fib Cairo Native: {:?}", ret)
    }

    pub fn execute_factorial(&self, n: Vec<u32>) -> String {
        let ret = execute_cairo_native_program(
            &self.fact_program,
            "fact_contract::fact_contract::Factorial::fact",
            vec![n],
            false,
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
    program: &Program,
    entry_point: &str,
    args: Vec<Vec<u32>>,
    include_syscall_handler: bool,
) -> Value {
    let mut writer: Vec<u8> = Vec::new();
    let mut res = serde_json::Serializer::new(&mut writer);
    // use params variable that is a deserializable variable
    let entry_point = match program
        .funcs
        .iter()
        .find(|x| x.id.debug_name.as_deref() == Some(entry_point))
    {
        Some(x) => x,
        None => {
            panic!("No entry point found");
        }
    };

    // Compile the program.
    let (context, mut module, registry, mut metadata) =
        cairo_native::easy::create_compiler(&program).unwrap();

    // Make the Starknet syscall handler available.
    metadata
        .insert(SyscallHandlerMeta::new(&syscall_handler::SyscallHandler))
        .unwrap();

    let required_initial_gas =
        cairo_native::easy::get_required_initial_gas(&program, &mut metadata, entry_point);

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
    cairo_native::easy::run_passes(&context, &mut module).unwrap();

    // Create the JIT engine.
    let engine = cairo_native::easy::create_engine(&module);

    let params_input = if include_syscall_handler {
        let system = metadata
            .get::<SyscallHandlerMeta>()
            .unwrap()
            .as_ptr()
            .addr();
        // pedersen, range check, gas, system
        json!([null, null, u64::MAX, system, [args]])
    } else {
        // _, gas, params
        let mut params = json!([null, u64::MAX]);
        for arg in args {
            params.as_array_mut().unwrap().push(arg.into());
        }
        params
    };

    let _ = cairo_native::execute(
        &engine,
        &registry,
        &entry_point.id,
        params_input,
        &mut res,
        required_initial_gas,
    )
    .unwrap();

    // The output expected as a string will be a json that looks like this:
    // [null,9000,[0,[[55,0,0,0,0,0,0,0]]]]
    let deserialized_result: String = String::from_utf8(writer).unwrap();
    let deserialized_value = serde_json::from_str::<serde_json::Value>(&deserialized_result)
        .expect("Failed to deserialize result");
    println!("{}", deserialized_value);

    deserialized_value
}

#[cfg(test)]
mod test {
    use std::{fs, io, path::Path, sync::Arc};

    use cairo_lang_compiler::CompilerConfig;
    use cairo_lang_sierra::{
        extensions::core::{CoreLibfunc, CoreType},
        ProgramParser,
    };
    use cairo_native::metadata::syscall_handler::SyscallHandlerMeta;
    use serde_json::json;

    use crate::cairo_native_engine::{execute_cairo_native_program, get_input_value_cairo_native};

    use super::syscall_handler;

    #[test]
    fn fib_10_cairo_native() {
        let a = super::get_input_value_cairo_native(0_usize);

        let b = super::get_input_value_cairo_native(1_usize);

        let n = super::get_input_value_cairo_native(10_usize);

        let sierra_program = cairo_lang_compiler::compile_cairo_project_at_path(
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
            false,
        );
        assert_eq!(fib_10[2][1][0][0].as_u64().unwrap(), 55);
    }

    #[test]
    fn compile_erc20_cairo_native() {
        let program_src = fs::read_to_string("../cairo_programs/erc20.sierra").unwrap();
        let program_parser = ProgramParser::new();
        let program = program_parser.parse(&program_src).unwrap();
        let entry_point = cairo_native::easy::find_entry_point(
            &program,
            "erc20::erc20::erc_20::__constructor::constructor",
        )
        .unwrap();

        // Compile the program.
        let (context, mut module, registry, mut metadata) =
            cairo_native::easy::create_compiler(&program).unwrap();

        // Make the Starknet syscall handler available.
        metadata
            .insert(SyscallHandlerMeta::new(&syscall_handler::SyscallHandler))
            .unwrap();

        let required_initial_gas =
            cairo_native::easy::get_required_initial_gas(&program, &mut metadata, entry_point);

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
        cairo_native::easy::run_passes(&context, &mut module).unwrap();

        // Create the JIT engine.
        let engine = cairo_native::easy::create_engine(&module);

        let params_input = json!([
            // pedersen
            null,
            // range check
            null,
            // gas
            u64::MAX,
            // system
            metadata
                .get::<SyscallHandlerMeta>()
                .unwrap()
                .as_ptr()
                .addr(),
            // The amount of params change depending on the contract function called
            // Struct<Span<Array<felt>>>
            [
                // Span<Array<felt>>
                [
                    // contract state

                    // name
                    cairo_native::easy::felt252_short_str("name"), // name
                    cairo_native::easy::felt252_short_str("symbol"), // symbol
                    cairo_native::easy::felt252_bigint(0),         // decimals
                    cairo_native::easy::felt252_bigint(i64::MAX),  // initial supply
                    cairo_native::easy::felt252_bigint(4),         // contract address
                    cairo_native::easy::felt252_bigint(6),         // ??
                ]
            ]
        ]);

        cairo_native::execute(
            &engine,
            &registry,
            &entry_point.id,
            params_input,
            &mut serde_json::Serializer::pretty(io::stdout()),
            required_initial_gas,
        )
        .unwrap();
    }

    #[test]
    fn erc20_cairo_native() {
        let program_src = fs::read_to_string("../cairo_programs/erc20.sierra").unwrap();
        let program = Arc::new(ProgramParser::new().parse(&program_src).unwrap());

        let _output = execute_cairo_native_program(
            &program,
            "erc20::erc20::erc_20::__constructor::constructor",
            vec![
                cairo_native::easy::felt252_short_str("name").to_vec(), // name
                cairo_native::easy::felt252_short_str("symbol").to_vec(), // symbol
                cairo_native::easy::felt252_bigint(0).to_vec(),         // decimals
                cairo_native::easy::felt252_bigint(1024).to_vec(),  // initial supply
                cairo_native::easy::felt252_bigint(4).to_vec(),         // contract address
                cairo_native::easy::felt252_bigint(6).to_vec(),         // ??
            ],
            true,
        );
    }
    #[test]
    fn fact_10_cairo_native() {
        let n = super::get_input_value_cairo_native(10_usize);

        let sierra_program = cairo_lang_compiler::compile_cairo_project_at_path(
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
            false,
        );
        assert_eq!(fact_10[2][1][0][0].as_u64().unwrap(), 3628800);
    }

    #[test]
    fn get_input_value_cairo_native_should_be_10() {
        let input = get_input_value_cairo_native(10);
        assert_eq!(input, vec![10, 0, 0, 0, 0, 0, 0, 0]);
    }
}

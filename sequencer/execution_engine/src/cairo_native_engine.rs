use cairo_lang_sierra::extensions::core::{CoreLibfunc, CoreType};
use cairo_lang_sierra::program::Program as SierraProgram;
use cairo_native::easy::compile_and_execute;
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

#[cfg(test)]
mod test {
    use std::path::Path;

    use cairo_lang_compiler::{compile_cairo_project_at_path, CompilerConfig};

    use crate::cairo_native_engine::{execute_cairo_native_program, get_input_value_cairo_native};

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

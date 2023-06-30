use lambdaworks_math::traits::Serializable;
use lambdaworks_stark::cairo::runner::run::{generate_prover_args, CairoVersion};
use lambdaworks_stark::starks::prover::prove;
use rustler::Binary;

const GRINDING_FACTOR: u8 = 10;

#[rustler::nif]
/// Loads the program in path, runs it with the Cairo VM, and makes a proof of it
pub fn run_program_and_get_proof_from_path(program_content_binary: Binary) -> Vec<u8> {
    let program_content = &program_content_binary as &[u8];
    run_program_and_get_proof_internal(program_content)
}

pub fn run_program_and_get_proof_internal(program_content: &[u8]) -> Vec<u8> {
    let (main_trace, cairo_air, mut pub_inputs) =
        generate_prover_args(program_content, &CairoVersion::V1, &None, GRINDING_FACTOR).unwrap();

    let proof = prove(&main_trace, &cairo_air, &mut pub_inputs).unwrap();
    let ret: Vec<u8> = proof.serialize();
    ret
}

#[cfg(test)]
mod test {
    #[test]
    fn test_run_program_and_get_proof() {
        let program_content = std::fs::read("../../programs/fibonacci_cairo1.casm").unwrap();
        let ret = super::run_program_and_get_proof_internal(&program_content);
        println!("ret len: {}", ret.len());
    }
}

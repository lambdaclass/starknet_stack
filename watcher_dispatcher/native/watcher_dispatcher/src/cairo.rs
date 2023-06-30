use lambdaworks_math::traits::Serializable;
use lambdaworks_stark::cairo::runner::run::{generate_prover_args, CairoVersion};
use lambdaworks_stark::starks::prover::prove;
use rustler::Binary;

const GRINDING_FACTOR: u8 = 10;

#[rustler::nif]
// Loads the program in path, runs it with the Cairo VM, and makes a proof of it
pub fn run_program_and_get_proof_from_path(file_path: Binary) -> Vec<u8> {
    let file_path = std::str::from_utf8(file_path.as_slice()).unwrap();
    run_program_and_get_proof_internal(file_path)
}

pub fn run_program_and_get_proof_internal(file_path: &str) -> Vec<u8> {
    let (main_trace, cairo_air, mut pub_inputs) =
        generate_prover_args(file_path, &CairoVersion::V1, &None, GRINDING_FACTOR).unwrap();

    let proof = prove(&main_trace, &cairo_air, &mut pub_inputs).unwrap();
    let ret: Vec<u8> = proof.serialize();
    ret
}

#[cfg(test)]
mod test {
    #[test]
    fn test_run_program_and_get_proof() {
        let ret = super::run_program_and_get_proof_internal("../../programs/fibonacci_cairo1.casm");
        println!("ret len: {}", ret.len());
    }
}

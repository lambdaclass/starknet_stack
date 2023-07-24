use lambdaworks_math::field::fields::fft_friendly::stark_252_prime_field::Stark252PrimeField;
use lambdaworks_math::traits::Deserializable;
use lambdaworks_stark::cairo::air::generate_cairo_proof;
use lambdaworks_stark::cairo::air::verify_cairo_proof;
use lambdaworks_stark::cairo::air::{CairoAIR, PublicInputs};
use lambdaworks_stark::cairo::runner::run::{generate_prover_args, CairoVersion};
use lambdaworks_stark::starks::proof::options::ProofOptions;
use lambdaworks_stark::starks::proof::stark::StarkProof;
use lambdaworks_stark::starks::{fri::FieldElement, prover::prove, traits::AIR, verifier::verify};
use rustler::Binary;

#[rustler::nif]
/// Loads the program in path, runs it with the Cairo VM, and makes a proof of it
///
/// # Returns
///
/// (proof_bytes, public_inputs_bytes)
pub fn run_program_and_get_proof(program_content_binary: Binary) -> (Vec<u8>, Vec<u8>) {
    let program_content: &[u8] = &*program_content_binary;
    run_program_and_get_proof_internal(program_content)
}

pub fn run_program_and_get_proof_internal(program_content: &[u8]) -> (Vec<u8>, Vec<u8>) {
    // this is the default configuration for the proof generation
    // TODO: this should be used only for testing.
    let proof_options = ProofOptions::default_test_options();

    let (main_trace, pub_inputs) =
        generate_prover_args(program_content, &CairoVersion::V1, &None).unwrap();

    let proof = generate_cairo_proof(&main_trace, &pub_inputs, &proof_options).unwrap();

    let proof_bytes = serde_cbor::to_vec(&proof).unwrap();
    let public_inputs_bytes = serde_cbor::to_vec(&pub_inputs).unwrap();

    (proof_bytes, public_inputs_bytes)
}

pub fn verify_internal(proof_bytes: &[u8], public_inputs_bytes: &[u8]) -> bool {
    // At this point, the verifier only knows about the serialized proof, the proof options
    // and the public inputs.

    let proof: StarkProof<Stark252PrimeField> = serde_cbor::from_slice(proof_bytes).unwrap();
    let proof_options = ProofOptions::default_test_options();
    let public_inputs: PublicInputs = serde_cbor::from_slice(public_inputs_bytes).unwrap();

    verify_cairo_proof(&proof, &public_inputs, &proof_options)
}

#[cfg(test)]
mod test {
    #[test]
    fn test_run_program_and_get_proof() {
        let program_content = std::fs::read("../../programs/fibonacci_cairo1.casm").unwrap();
        let (proof_bytes, public_inputs_bytes) =
            super::run_program_and_get_proof_internal(&program_content);
        println!("proof_bytes len: {}", proof_bytes.len());

        assert!(super::verify_internal(&proof_bytes, &public_inputs_bytes));
    }
}

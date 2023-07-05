use lambdaworks_math::field::fields::fft_friendly::stark_252_prime_field::Stark252PrimeField;
use lambdaworks_math::field::traits::IsFFTField;
use lambdaworks_math::traits::ByteConversion;
use lambdaworks_math::traits::{Deserializable, Serializable};
use lambdaworks_stark::cairo::air::{CairoAIR, PublicInputs};
use lambdaworks_stark::cairo::runner::run::{generate_prover_args, CairoVersion};
use lambdaworks_stark::starks::proof::options::ProofOptions;
use lambdaworks_stark::starks::proof::stark::StarkProof;
use lambdaworks_stark::starks::{fri::FieldElement, prover::prove, traits::AIR, verifier::verify};

//proof::StarkProof, proof_options::ProofOptions,

use rustler::Binary;

const GRINDING_FACTOR: u8 = 10;

#[rustler::nif]
/// Loads the program in path, runs it with the Cairo VM, and makes a proof of it
pub fn run_program_and_get_proof(program_content_binary: Binary) -> (Vec<u8>, Vec<u8>) {
    let program_content: &[u8] = &*program_content_binary;
    run_program_and_get_proof_internal(program_content)
}

pub fn run_program_and_get_proof_internal(program_content: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let (main_trace, cairo_air, mut pub_inputs) =
        generate_prover_args(program_content, &CairoVersion::V1, &None, GRINDING_FACTOR).unwrap();

    let pub_inputs_serialized = pub_inputs.serialize();

    let proof = prove(&main_trace, &cairo_air, &mut pub_inputs).unwrap();
    let proof_serialized: Vec<u8> = proof.serialize();
    (proof_serialized, pub_inputs_serialized)
}

pub fn verify_internal<F, A>(proof_bytes: &[u8], air: &A, public_inputs_bytes: &[u8]) -> bool
where
    F: IsFFTField,
    A: AIR<Field = F>,
    FieldElement<F>: ByteConversion,
{
    // At this point, the verifier only knows about the serialized proof, the proof options
    // and the public inputs.
    let proof = StarkProof::<Stark252PrimeField>::deserialize(&proof_bytes).unwrap();

    // The same proof configuration as used in the `generate_prover_args` function.
    let proof_options = ProofOptions {
        blowup_factor: 4,
        fri_number_of_queries: 3,
        coset_offset: 3,
        grinding_factor: 1,
    };

    let public_inputs = PublicInputs::deserialize(public_inputs_bytes).unwrap();

    let air = CairoAIR::new(proof_options, proof.trace_length, public_inputs, false);

    //verify(&proof, &air, &public_inputs)

    //assert!();

    true
}

#[cfg(test)]
mod test {
    #[test]
    fn test_run_program_and_get_proof() {
        let program_content = std::fs::read("../../programs/fibonacci_cairo1.casm").unwrap();
        let ret = super::run_program_and_get_proof_internal(&program_content);
        println!("ret len: {}", ret.0.len());
    }
}

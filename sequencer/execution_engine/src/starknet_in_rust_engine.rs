use std::{collections::HashMap, sync::Arc};

use cairo_felt::{felt_str, Felt252};
use starknet_in_rust::{
    core::contract_address::compute_deprecated_class_hash,
    definitions::{
        block_context::{BlockContext, StarknetChainId},
        constants::EXECUTE_ENTRY_POINT_SELECTOR,
    },
    services::api::contract_classes::deprecated_contract_class::ContractClass,
    simulate_transaction,
    state::{cached_state::CachedState, in_memory_state_reader::InMemoryStateReader},
    transaction::{InvokeFunction, Transaction},
    utils::{felt_to_hash, Address, ClassHash},
    CasmContractClass,
};

pub struct StarknetState {
    state: CachedState<InMemoryStateReader>,
    fib_entrypoint_selector: Felt252,
    fact_entrypoint_selector: Felt252,
}

impl StarknetState {
    pub fn new_for_tests() -> Self {
        let program_data = include_bytes!("../../cairo_programs/fib_contract.casm");
        let contract_class: CasmContractClass = serde_json::from_slice(program_data).unwrap();

        let mut contract_class_cache = HashMap::new();

        let address = Address(0.into());
        let class_hash: ClassHash = [1; 32];

        contract_class_cache.insert(class_hash, contract_class.clone());
        let mut state_reader = InMemoryStateReader::default();

        let entrypoints = contract_class.entry_points_by_type;
        let fib_entrypoint_selector = &entrypoints.external.get(0).unwrap().selector;

        state_reader
            .address_to_class_hash_mut()
            .insert(address.clone(), class_hash);

        state_reader
            .address_to_nonce_mut()
            .insert(address.clone(), Felt252::new(0));

        // pre-add factorial
        {
            let program_data = include_bytes!("../../cairo_programs/fact_contract.casm");
            let contract_class: CasmContractClass = serde_json::from_slice(program_data).unwrap();

            let entrypoints = contract_class.clone().entry_points_by_type;
            let fact_entrypoint_selector = &entrypoints.external.get(0).unwrap().selector;

            let class_hash: ClassHash = [2; 32];
            contract_class_cache.insert(class_hash, contract_class);

            state_reader
                .address_to_class_hash_mut()
                .insert(Address(1.into()).clone(), class_hash);

            state_reader
                .address_to_nonce_mut()
                .insert(Address(1.into()).clone(), Felt252::new(0));
        }

        {
            // data to deploy
            let erc20_class_hash: ClassHash = [2; 32];
            let test_data = include_bytes!("../../cairo_programs/erc20.casm");
            let test_contract_class: CasmContractClass = serde_json::from_slice(test_data).unwrap();

            let entrypoints = test_contract_class.clone().entry_points_by_type;
            let entrypoint_selector = &entrypoints.external.get(0).unwrap().selector;

            let address = Address(2.into());
            let class_hash: ClassHash = [3; 32];
            let nonce = Felt252::new(0);

            //contract_class_cache.insert(class_hash, class_hash);
            //contract_class_cache.insert(erc20_class_hash, test_contract_class);

            let mut state_reader = InMemoryStateReader::default();
            state_reader
                .address_to_class_hash_mut()
                .insert(address.clone(), class_hash);

            state_reader
                .address_to_nonce_mut()
                .insert(address.clone(), nonce);

            // let name_ = Felt252::from_bytes_be(b"some-token");
            // let symbol_ = Felt252::from_bytes_be(b"my-super-awesome-token");
            // let decimals_ = Felt252::from(24);
            // let initial_supply = Felt252::from(1000);
            // let recipient =
            //     felt_str!("397149464972449753182583229366244826403270781177748543857889179957856017275");
            // let erc20_salt = felt_str!("1234");
            // // arguments of deploy contract
            // let calldata = vec![
            //     Felt252::from_bytes_be(&erc20_class_hash),
            //     erc20_salt,
            //     recipient,
            //     name_,
            //     decimals_,
            //     initial_supply,
            //     symbol_,
            // ];
        }


        let state = CachedState::new(Arc::new(state_reader), None, Some(contract_class_cache));

        StarknetState {
            state,
            fib_entrypoint_selector: fib_entrypoint_selector.into(),
            fact_entrypoint_selector: 222.into(),
        }
    }

    pub fn execute_fibonacci(&mut self, calldata: Vec<Felt252>) -> Vec<Felt252> {
        let invoke_tx = InvokeFunction::new(
            Address(0.into()),
            self.fib_entrypoint_selector.clone(),
            u128::MAX,
            Felt252::new(0),
            calldata,
            vec![],
            Felt252::new(0),
            None,
        )
        .unwrap();

        let return_data = invoke_tx
            .create_for_simulation(false, false, true, true)
            .execute(&mut self.state, &BlockContext::default(), u128::MAX)
            .unwrap();
        return_data.call_info.unwrap().retdata
    }

    pub fn execute_factorial(&self, n: usize) -> String {
        format!("Output Fact Cairo VM: {:?}", "res")
    }
}

#[cfg(test)]
mod tests {
    use cairo_felt::Felt252;
    use starknet_in_rust::{
        call_contract, definitions::block_context::BlockContext, transaction::InvokeFunction,
        utils::Address, CasmContractClass,
    };

    use crate::starknet_in_rust_engine::StarknetState;

    #[test]
    fn execute_fibonacci() {
        let mut starknet_state = StarknetState::new_for_tests();

        let fib_of_10 = starknet_state.execute_fibonacci(vec![1.into(), 1.into(), 10.into()]);
        assert_eq!(fib_of_10, vec![89.into()]);
    }

    #[test]
    fn call_contract_fibonacci_with_10_should_return_89() {
        let mut starknet_state = StarknetState::new_for_tests();

        let calldata = [1.into(), 1.into(), 10.into()].to_vec();

        let retdata = call_contract(
            0.into(),
            starknet_state.fib_entrypoint_selector.clone(),
            calldata.clone(),
            &mut starknet_state.state,
            BlockContext::default(),
            Address(0.into()),
        )
        .unwrap();

        let invoke_tx = InvokeFunction::new(
            Address(0.into()),
            starknet_state.fib_entrypoint_selector.clone(),
            u128::MAX,
            Felt252::new(0),
            calldata,
            vec![],
            Felt252::new(0),
            None,
        )
        .unwrap();

        let invoke_tx = invoke_tx.create_for_simulation(false, false, true, true);

        let res = invoke_tx.execute(
            &mut starknet_state.state,
            &BlockContext::default(),
            u128::MAX,
        );

        assert_eq!(retdata, vec![89.into()]);
        assert_eq!(res.unwrap().call_info.unwrap().retdata, vec![89.into()]);
    }
}

use cairo_felt::Felt252;
use starknet_in_rust::{
    definitions::block_context::BlockContext,
    execution::TransactionExecutionInfo,
    state::{cached_state::CachedState, in_memory_state_reader::InMemoryStateReader},
    transaction::{error::TransactionError, InvokeFunction, Transaction},
    utils::{Address, ClassHash},
};
use std::{collections::HashMap, sync::Arc};

pub struct StarknetState {
    state: CachedState<InMemoryStateReader>,
    block_context: BlockContext,
}

impl StarknetState {
    pub fn new_for_tests() -> Self {
        let mut state_reader = InMemoryStateReader::default();
        // Create state reader with class hash data

        let mut sierra_contract_class_cache = HashMap::new();

        {
            let contract_address = Address(0.into());
            let class_hash: ClassHash = [1; 32];

            let program_data = include_bytes!("../../cairo_programs/cairo2/fibonacci.sierra");
            let contract_class: cairo_lang_starknet::contract_class::ContractClass =
                serde_json::from_slice(program_data).unwrap();

            sierra_contract_class_cache.insert(class_hash, contract_class.clone());

            state_reader
                .address_to_class_hash_mut()
                .insert(contract_address.clone(), class_hash);

            state_reader
                .address_to_nonce_mut()
                .insert(contract_address, Felt252::new(0));
        };
        // pre-add factorial
        {
            let program_data: &[u8] =
                include_bytes!("../../cairo_programs/cairo2/factorial.sierra");
            let contract_class: cairo_lang_starknet::contract_class::ContractClass =
                serde_json::from_slice(program_data).unwrap();

            let _entrypoints = contract_class.clone().entry_points_by_type;
            let contract_address = Address(1.into());
            let class_hash: ClassHash = [2; 32];
            sierra_contract_class_cache.insert(class_hash, contract_class);

            state_reader
                .address_to_class_hash_mut()
                .insert(contract_address.clone(), class_hash);

            state_reader
                .address_to_nonce_mut()
                .insert(contract_address, Felt252::new(0));
        };

        {
            // data to deploy
            let contract_address = Address(2.into());
            let erc20_class_hash: ClassHash = [3; 32];

            let sierra_contract_class: cairo_lang_starknet::contract_class::ContractClass =
                serde_json::from_slice(include_bytes!("../../cairo_programs/cairo2/erc20.sierra"))
                    .unwrap();

            sierra_contract_class_cache.insert(erc20_class_hash, sierra_contract_class);

            // contract_class_cache.insert(erc20_class_hash, test_contract_class.clone());

            let nonce = Felt252::new(0);

            //contract_class_cache.insert(class_hash, class_hash);
            //contract_class_cache.insert(erc20_class_hash, test_contract_class);

            state_reader
                .address_to_class_hash_mut()
                .insert(contract_address.clone(), erc20_class_hash);

            state_reader
                .address_to_nonce_mut()
                .insert(contract_address, nonce);
        };

        let state = CachedState::new(Arc::new(state_reader))
            .set_sierra_programs_cache(sierra_contract_class_cache);

        StarknetState {
            state,
            block_context: BlockContext::default(),
        }
    }

    pub fn invoke(&mut self, calldata: Vec<Felt252>) -> Result<Vec<Felt252>, TransactionError> {
        let invoke_tx_execution = InvokeFunction::new(
            Address(
                calldata
                    .first()
                    .expect("Invoke does not contain contract address")
                    .clone(),
            ),
            calldata
                .get(1)
                .expect("Invoke does not contain function selector")
                .clone(),
            u128::MAX,
            Felt252::new(0),
            calldata[2..].to_vec(),
            vec![],
            Felt252::new(0),
            None,
        )
        .and_then(|invoke_tx| {
            let return_data = invoke_tx
                .create_for_simulation(true, false, true, true, true)
                .execute(&mut self.state, &self.block_context, u128::MAX)?;

            return_data
                .call_info
                .ok_or(TransactionError::CallInfoIsNone)
                .map(|x| x.retdata)
        });

        invoke_tx_execution
    }

    pub fn handle_transaction(
        &mut self,
        tx: Transaction,
    ) -> Result<TransactionExecutionInfo, TransactionError> {
        tx.create_for_simulation(true, false, true, false, false)
            .execute(&mut self.state, &self.block_context, u128::MAX)
    }
}

#[cfg(test)]
mod tests {
    use crate::starknet_in_rust_engine::StarknetState;
    use cairo_felt::{felt_str, Felt252};

    #[test]
    fn factorial_test() {
        let mut starknet_state = StarknetState::new_for_tests();

        // valid fact call
        let selector = felt_str!(
            "36fbc999025b89d36d31dc2f9c0a03b4377755e1f27e0e42a385aaba90f61a6",
            16
        );

        starknet_state
            .invoke(vec![Felt252::new(1), selector, Felt252::new(2000)])
            .unwrap();
    }

    #[test]
    fn fibonacci_test() {
        let mut starknet_state = StarknetState::new_for_tests();
        // valid fib call
        let selector = felt_str!(
            "112e35f48499939272000bd72eb840e502ca4c3aefa8800992e8defb746e0c9",
            16
        );

        starknet_state
            .invoke(vec![
                Felt252::new(0),
                selector,
                Felt252::new(1),
                Felt252::new(1),
                Felt252::new(10000),
            ])
            .unwrap();

        // NOTE: this is commented out for now because the Cairo Native branch of the execution panics when an error like this happens
        // should fail due to bad selector
        //let selector = felt_str!("abb", 16);
        //starknet_state
        //    .invoke(vec![Felt252::new(0), selector, Felt252::new(1), Felt252::new(1), Felt252::new(10000)])
        //    .unwrap_err();
    }

    #[test]
    fn test_erc20() {
        let mut starknet_state = StarknetState::new_for_tests();

        // valid erc20 call
        let selector = felt_str!(
            "83afd3f4caedc6eebf44246fe54e38c95e3179a5ec9ea81740eca5b482d12e",
            16
        );

        let _initial_supply = Felt252::new((5000) + 1);
        let name = Felt252::new(256);
        let token_symbol = Felt252::new(512);
        let _decimals = Felt252::new(10);
        let _contract_address_receiver = Felt252::new(10);

        // execution type felt, initial_supply, token symbol, contract address
        starknet_state
            .invoke(vec![
                Felt252::new(2),
                selector,
                name,
                token_symbol,
                //decimals,
                //initial_supply,
                //contract_address_receiver,
                //Felt252::new(6),
            ])
            .unwrap();

        // should fail due to not deployed contract
        let selector = felt_str!(
            "213cda0181d4bd6d07f2e467ddf45a1d971e14ca1bcd4c83949a6d830a15b7f",
            16
        );
        starknet_state
            .invoke(vec![Felt252::new(9999), selector, Felt252::new(2000)])
            .unwrap_err();
    }

    #[test]
    fn test_not_deployed_contract() {
        let mut starknet_state = StarknetState::new_for_tests();

        // should fail due to not deployed contract
        let selector = felt_str!(
            "213cda0181d4bd6d07f2e467ddf45a1d971e14ca1bcd4c83949a6d830a15b7f",
            16
        );

        starknet_state
            .invoke(vec![Felt252::new(9999), selector, Felt252::new(2000)])
            .unwrap_err();
    }
}

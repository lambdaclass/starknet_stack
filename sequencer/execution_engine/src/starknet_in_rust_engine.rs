use cairo_felt::{felt_str, Felt252};
use starknet_in_rust::{
    definitions::{block_context::BlockContext, transaction_type::TransactionType, constants::VALIDATE_DECLARE_ENTRY_POINT_SELECTOR},
    state::{cached_state::CachedState, in_memory_state_reader::InMemoryStateReader},
    transaction::{error::TransactionError, InvokeFunction, DeclareV2, declare, Transaction},
    utils::{Address, ClassHash},
    CasmContractClass, core::contract_address::{compute_sierra_class_hash, compute_casm_class_hash}, testing::TEST_ACCOUNT_CONTRACT_ADDRESS, execution::TransactionExecutionInfo,
};
use std::{collections::HashMap, sync::Arc};
use cairo_lang_starknet::contract_class::ContractClass as SierraContractClass;

pub struct StarknetState {
    state: CachedState<InMemoryStateReader>,
    block_context: BlockContext
}

impl StarknetState {
    pub fn new_for_tests() -> Self {
        let mut contract_class_cache = HashMap::new();
        let mut state_reader = InMemoryStateReader::default();

        let _fib_entrypoint_selector = {
            let contract_address = Address(0.into());
            let class_hash: ClassHash = [1; 32];

            let program_data = include_bytes!("../../cairo_programs/fib_contract.casm");
            let contract_class: CasmContractClass = serde_json::from_slice(program_data).unwrap();

            contract_class_cache.insert(class_hash, contract_class.clone());

            state_reader
                .address_to_class_hash_mut()
                .insert(contract_address.clone(), class_hash);

            state_reader
                .address_to_nonce_mut()
                .insert(contract_address, Felt252::new(0));

            contract_class
                .entry_points_by_type
                .external
                .get(0)
                .unwrap()
                .selector
                .clone()
        };
        // pre-add factorial
        let _fact_entrypoint_selector = {
            let program_data: &[u8] = include_bytes!("../../cairo_programs/fact_contract.casm");
            let contract_class: CasmContractClass = serde_json::from_slice(program_data).unwrap();

            let entrypoints = contract_class.clone().entry_points_by_type;
            let contract_address = Address(1.into());
            let class_hash: ClassHash = [2; 32];
            contract_class_cache.insert(class_hash, contract_class);

            state_reader
                .address_to_class_hash_mut()
                .insert(contract_address.clone(), class_hash);

            state_reader
                .address_to_nonce_mut()
                .insert(contract_address, Felt252::new(0));

            entrypoints.external.get(0).unwrap().selector.clone()
        };

        let _erc20_entrypoint_selector = {
            // data to deploy
            let contract_address = Address(2.into());
            let erc20_class_hash: ClassHash = [3; 32];
            let test_data = include_bytes!("../../cairo_programs/erc20.casm");
            let test_contract_class: CasmContractClass = serde_json::from_slice(test_data).unwrap();

            contract_class_cache.insert(erc20_class_hash, test_contract_class.clone());

            let nonce = Felt252::new(0);

            //contract_class_cache.insert(class_hash, class_hash);
            //contract_class_cache.insert(erc20_class_hash, test_contract_class);

            state_reader
                .address_to_class_hash_mut()
                .insert(contract_address.clone(), erc20_class_hash);

            state_reader
                .address_to_nonce_mut()
                .insert(contract_address, nonce);

            test_contract_class
                .entry_points_by_type
                .external
                .get(0)
                .unwrap()
                .selector
                .clone()
        };

        let state = CachedState::new(Arc::new(state_reader), None, Some(contract_class_cache));

        StarknetState { state,
        block_context: BlockContext::default() }
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
            calldata[1..].to_vec(),
            vec![],
            Felt252::new(0),
            None,
        )
        .and_then(|invoke_tx| {
            let return_data = invoke_tx
                .create_for_simulation(false, false, true, true)
                .execute(&mut self.state, &self.block_context, u128::MAX)?;

            return_data
                .call_info
                .ok_or(TransactionError::CallInfoIsNone)
                .map(|x| x.retdata)
        });

        invoke_tx_execution
    }

    pub fn declare_v2(&mut self) -> Result<TransactionExecutionInfo, TransactionError> {
        let program_data = &[0];
        let sierra_contract_class: SierraContractClass = serde_json::from_slice(program_data).unwrap();
        let sierra_class_hash = compute_sierra_class_hash(&sierra_contract_class).unwrap();
        let casm_class =
            CasmContractClass::from_contract_class(sierra_contract_class.clone(), true).unwrap();
        let casm_class_hash = compute_casm_class_hash(&casm_class).unwrap();
    
        let declare_tx = DeclareV2 {
            sender_address: TEST_ACCOUNT_CONTRACT_ADDRESS.clone(),
            tx_type: TransactionType::Declare,
            validate_entry_point_selector: VALIDATE_DECLARE_ENTRY_POINT_SELECTOR.clone(),
            version: 1.into(),
            max_fee: 50000000,
            signature: vec![],
            nonce: 0.into(),
            hash_value: 0.into(),
            compiled_class_hash: casm_class_hash,
            sierra_contract_class,
            sierra_class_hash,
            casm_class: casm_class.into(),
            skip_execute: false,
            skip_fee_transfer: false,
            skip_validate: false,
        };

        declare_tx.execute(&mut self.state, &self.block_context)
    }

    pub fn execute_transaction(&mut self, tx: Transaction) -> Result<TransactionExecutionInfo, TransactionError> {
        tx.create_for_simulation(true, false, true, false).execute(&mut self.state, &self.block_context, u128::MAX)
    }
}

#[cfg(test)]
mod tests {
    use crate::starknet_in_rust_engine::StarknetState;
    use cairo_felt::{felt_str, Felt252};

    #[test]
    fn test_three_contracts() {
        let mut starknet_state = StarknetState::new_for_tests();

        // valid fib call
        let selector = felt_str!(
            "112e35f48499939272000bd72eb840e502ca4c3aefa8800992e8defb746e0c9",
            16
        );
        starknet_state
            .invoke(vec![Felt252::new(0), selector, Felt252::new(10000)])
            .unwrap();

        // should fail due to bad selector
        let selector = felt_str!("abb", 16);
        starknet_state
            .invoke(vec![Felt252::new(0), selector, Felt252::new(10000)])
            .unwrap_err();

        // valid fact call
        let selector = felt_str!(
            "213cda0181d4bd6d07f2e467ddf45a1d971e14ca1bcd4c83949a6d830a15b7f",
            16
        );
        starknet_state
            .invoke(vec![Felt252::new(1), selector, Felt252::new(2000)])
            .unwrap();

        // valid erc20 call
        let selector = felt_str!(
            "83afd3f4caedc6eebf44246fe54e38c95e3179a5ec9ea81740eca5b482d12e",
            16
        );
        let initial_supply = Felt252::new((5000) + 1);
        let token_symbol = Felt252::new(512);
        let contract_address_receiver = Felt252::new(10);
        // execution type felt, initial_supply, token symbol, contract address
        starknet_state
            .invoke(vec![
                Felt252::new(2),
                selector,
                initial_supply,
                token_symbol,
                contract_address_receiver,
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
}

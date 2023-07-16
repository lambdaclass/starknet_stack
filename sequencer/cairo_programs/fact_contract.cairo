#[starknet::contract]
mod Factorial{

    #[external]
    fn fact(n: felt252) -> felt252 {
        match n {
            0 => 1,
            _ => n * fact(n - 1)
        }
    }
}
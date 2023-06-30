pub mod cairo;

pub use cairo::run_program_and_get_proof_from_path;

#[rustler::nif]
fn add(a: i64, b: i64) -> i64 {
    println!("Hello from Rust!");
    a + b
}

rustler::init!(
    "Elixir.WatcherDispatcher.NIF",
    [add, run_program_and_get_proof_from_path]
);

use mpz_circuits::{mpz_dynamic_types::composite::StaticCompositeType, Circuit};
use std::fs::write;

fn main() {
    build_aes();
    build_sha();
}

fn build_aes() {
    let circ = Circuit::parse(
        "circuits/bristol/aes_128_reverse.txt",
        &[<[u8; 16]>::TYPE, <[u8; 16]>::TYPE],
        &[<[u8; 16]>::TYPE],
    )
    .unwrap()
    .reverse_input(0)
    .reverse_input(1)
    .reverse_output(0);

    let bytes = bincode::serialize(&circ).unwrap();
    write("circuits/bin/aes_128.bin", bytes).unwrap();
}

fn build_sha() {
    let circ = Circuit::parse(
        "circuits/bristol/sha256_reverse.txt",
        &[<[u8; 64]>::TYPE, <[u32; 8]>::TYPE],
        &[<[u32; 8]>::TYPE],
    )
    .unwrap()
    .reverse_inputs()
    .reverse_input(0)
    .reverse_input(1)
    .reverse_output(0);

    let bytes = bincode::serialize(&circ).unwrap();
    write("circuits/bin/sha256.bin", bytes).unwrap();
}

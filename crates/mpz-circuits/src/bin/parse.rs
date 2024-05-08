use mpz_circuits::{Circuit, PrimitiveType, ValueType};
use std::fs::write;

fn main() {
    build_aes();
    build_sha();
}

fn build_aes() {
    let circ = Circuit::parse(
        "circuits/bristol/aes_128_reverse.txt",
        &[
            ValueType::Array {
                ty: PrimitiveType::U8,
                len: 16,
            },
            ValueType::Array {
                ty: PrimitiveType::U8,
                len: 16,
            },
        ],
        &[ValueType::Array {
            ty: PrimitiveType::U8,
            len: 16,
        }],
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
        &[
            ValueType::Array {
                ty: PrimitiveType::U8,
                len: 64,
            },
            ValueType::Array {
                ty: PrimitiveType::U32,
                len: 8,
            },
        ],
        &[ValueType::Array {
            ty: PrimitiveType::U32,
            len: 8,
        }],
    )
    .unwrap()
    .reverse_inputs()
    .reverse_input(0)
    .reverse_input(1)
    .reverse_output(0);

    let bytes = bincode::serialize(&circ).unwrap();
    write("circuits/bin/sha256.bin", bytes).unwrap();
}

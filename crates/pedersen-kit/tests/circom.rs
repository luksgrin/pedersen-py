//! Byte-compatibility with circomlib / circomlibjs (`pedersenHash`).
//!
//! Drives one reusable `BabyJubjubCircom` (memoized generators) against a table
//! of official `circomlibjs` outputs: empty input, sub-segment, exact/over
//! segment boundaries, and multi-segment inputs. The 200-byte case needs 8
//! segments, exercising the BLAKE-256 fallback beyond the baked table.
//!
//! Run with `--features circom`.
#![cfg(feature = "circom")]

use pedersen_kit::circom::BabyJubjubCircom;

fn input(len: usize) -> Vec<u8> {
    (0..len).map(|i| i as u8).collect()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// (input byte-length, circomlibjs `pedersen.hash` packed output). Input is the
/// byte sequence 0,1,…,len-1. Segments are 200 bits (25 bytes).
const VECTORS: &[(usize, &str)] = &[
    (
        0,
        "0100000000000000000000000000000000000000000000000000000000000000",
    ),
    (
        1,
        "4342ded81a9c9adc4472f5732febf9b1018ed754ccaf8f0ce9c5d09e6400e30d",
    ),
    (
        7,
        "7f7607a9d5428169a95baecff157fc5dd7de62ebf46083f85b2db56450c94313",
    ),
    (
        16,
        "0a4af928a3f75f38acce9e8e55b360ada01c4a2aa045aac64e8a7cb7499db108",
    ),
    (
        25,
        "1329a7ebe58a025ffddde4f9e3018caa4af839b0304579feb2355cb871590715",
    ),
    (
        26,
        "21ec02d20b056813a0bc784f0ced2a1a60104fb109425b5ab632235ce40edaa1",
    ),
    (
        32,
        "3b8b309e4979c8ad186a18c7895478e5e5f6dff59d2b91d3e71824cf7d5e3da1",
    ),
    (
        64,
        "58b7b97eb2fd6adb8e43a6ec24ee3c27a92bae7e6375a86f426d076d4b3ed826",
    ),
    (
        96,
        "8f2565e8c637853e59bf45a4f2afab342c0728160ed7cf0d6231272aafc9c286",
    ),
    (
        127,
        "c67cc1d9191586ac969cfda0395911df25e7689a9268ec2e2b149b652daa8625",
    ),
    // 200 bytes → 8 segments → uses baked bases 0..6 and BLAKE-256 for 6,7.
    (
        200,
        "e6ca0f8c73f0abcc6fa05234483be93a9b67b1f9275863961567fc719bb0be0e",
    ),
];

#[test]
fn matches_circomlibjs_vectors() {
    let mut hasher = BabyJubjubCircom::new(); // reused: exercises memoized growth
    for &(len, expected) in VECTORS {
        assert_eq!(
            hex(&hasher.hash(&input(len))),
            expected,
            "circomlibjs len {len}"
        );
    }
}

//! Byte-compatibility with circomlib / circomlibjs (`pedersenHash`).
//!
//! Drives the library's `circom` instance against a table of official
//! `circomlibjs` outputs spanning empty input, sub-segment, exact/over segment
//! boundaries, and multi-segment inputs reaching base points 0..=5.
//!
//! Run with `--features circom`.
#![cfg(feature = "circom")]

use pedersen_kit::circom::hasher_for_len;

fn input(len: usize) -> Vec<u8> {
    (0..len).map(|i| i as u8).collect()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// (input byte-length, circomlibjs `pedersen.hash` packed output).
/// Input is the byte sequence 0,1,2,…,len-1. Segments are 200 bits (25 bytes),
/// so lengths ≥ 26/51/76/101/126 reach base points 1/2/3/4/5.
const VECTORS: &[(usize, &str)] = &[
    (0, "0100000000000000000000000000000000000000000000000000000000000000"),
    (1, "4342ded81a9c9adc4472f5732febf9b1018ed754ccaf8f0ce9c5d09e6400e30d"),
    (7, "7f7607a9d5428169a95baecff157fc5dd7de62ebf46083f85b2db56450c94313"),
    (16, "0a4af928a3f75f38acce9e8e55b360ada01c4a2aa045aac64e8a7cb7499db108"),
    (25, "1329a7ebe58a025ffddde4f9e3018caa4af839b0304579feb2355cb871590715"),
    (26, "21ec02d20b056813a0bc784f0ced2a1a60104fb109425b5ab632235ce40edaa1"),
    (32, "3b8b309e4979c8ad186a18c7895478e5e5f6dff59d2b91d3e71824cf7d5e3da1"),
    (64, "58b7b97eb2fd6adb8e43a6ec24ee3c27a92bae7e6375a86f426d076d4b3ed826"),
    (96, "8f2565e8c637853e59bf45a4f2afab342c0728160ed7cf0d6231272aafc9c286"),
    (127, "c67cc1d9191586ac969cfda0395911df25e7689a9268ec2e2b149b652daa8625"),
];

#[test]
fn matches_circomlibjs_vectors() {
    for &(len, expected) in VECTORS {
        let hasher = hasher_for_len(len);
        assert_eq!(hex(&hasher.hash(&input(len))), expected, "circomlibjs len {len}");
    }
}

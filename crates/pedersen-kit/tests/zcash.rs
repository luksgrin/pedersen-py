//! Byte-compatibility with Zcash Sapling's Pedersen hash.
//!
//! Drives one reusable `JubjubSapling` (memoized generators) against a table of
//! outputs from `zcash/zcash-test-vectors`. The 200-byte case needs 9 segments,
//! exercising the BLAKE2s fallback beyond the baked table.
//!
//! Run with `--features zcash`.
#![cfg(feature = "zcash")]

use ark_ec::CurveGroup;
use ark_ed_on_bls12_381::Fq;
use core::str::FromStr;

use pedersen_kit::zcash::{JubjubSapling, ZCASH_PH, find_group_hash};

fn input(len: usize) -> Vec<u8> {
    (0..len).map(|i| i as u8).collect()
}

/// (input byte-length, Zcash `pedersen_hash` u-coordinate as decimal). Input is
/// the byte sequence 0,1,…,len-1. Segments are 189 bits (non-byte-aligned).
const VECTORS: &[(usize, &str)] = &[
    (0, "0"),
    (
        1,
        "18199356453276551058544483067971537764100958815371062302545925104228111306218",
    ),
    (
        7,
        "30099464267017412800553966313415301452778432059177165170366978351914608846776",
    ),
    (
        16,
        "12696241130161684714785648917475044367558106373835603479612743300999431757347",
    ),
    (
        25,
        "40252675167179038683265079502460445549985017473999381908939385819683359801490",
    ),
    (
        26,
        "1491362650589562623177075487200112341461920652079790862413814396101342312904",
    ),
    (
        32,
        "25017417399659973419738556147872271958000867140359490243475813708054159729475",
    ),
    (
        64,
        "37515569649653130145499701737487402729855929021425639231448526651152569150619",
    ),
    (
        96,
        "8119631812570225468889042567948308638876073095616697550291528740446269199445",
    ),
    (
        127,
        "5071382384466765121380455708522877243045923848243135694876540672176850044532",
    ),
    // 200 bytes → 9 segments → uses baked bases 0..6 and BLAKE2s for 6,7,8.
    (
        200,
        "42827919558462980191913516929316077916340907458449651636274249305684465418543",
    ),
];

#[test]
fn matches_zcash_vectors() {
    let mut hasher = JubjubSapling::new(); // reused: exercises memoized growth
    for &(len, expected) in VECTORS {
        assert_eq!(
            hasher.hash(&input(len)),
            Fq::from_str(expected).unwrap(),
            "zcash len {len}"
        );
    }
}

#[test]
fn generator_one_matches_reference() {
    // find_group_hash(D, i2leosp(32, 0)); confirms arkworks Jubjub (x, y) == Zcash (u, v).
    let g = find_group_hash(&ZCASH_PH, &[0, 0, 0, 0]).into_affine();
    assert_eq!(
        g.x,
        Fq::from_str(
            "52355368488200756720908213129543630848976972731871436319321443845291207170897"
        )
        .unwrap(),
    );
    assert_eq!(
        g.y,
        Fq::from_str(
            "18372611905088487385433946659983357101887954355879737496286092836680199584970"
        )
        .unwrap(),
    );
}

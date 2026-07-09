//! Byte-compatibility with circomlib / circomlibjs (`pedersenHash`).
//!
//! This reconstructs circom's real generators (BLAKE-256 find-group-hash) and its
//! `packPoint` output encoding — the two spec-specific pieces — and drives them
//! through the *library* skeleton (`Circom` encoding + `LsbFirst`), then checks
//! the result against the official `circomlibjs` test vector for `"Hello"`.
//!
//! The generator/packing reconstruction and BLAKE-256 are TEST-ONLY (dev-dep).

use ark_babyjubjub::{EdwardsAffine, EdwardsConfig, EdwardsProjective, Fq};
use ark_ec::{twisted_edwards::TECurveConfig, AdditiveGroup, AffineRepr};
use ark_ff::{BigInt, BigInteger, Field, PrimeField};
use blake_hash::{Blake256, Digest};

use pedersen_kit::{Circom, LsbFirst, Parameters, Pedersen, WholePoint};

/// babyjub `unpackPoint`: 32 LE bytes → point, or `None` if invalid.
fn unpack(mut bytes: [u8; 32]) -> Option<EdwardsAffine> {
    let sign = bytes[31] & 0x80 != 0;
    bytes[31] &= 0x7f;

    // y as an integer, rejected if >= field modulus (as circom does).
    let mut limbs = [0u64; 4];
    for (i, limb) in limbs.iter_mut().enumerate() {
        *limb = u64::from_le_bytes(bytes[i * 8..i * 8 + 8].try_into().unwrap());
    }
    let y = Fq::from_bigint(BigInt::new(limbs))?;

    // x² = (1 - y²) / (A - D·y²)
    let a = <EdwardsConfig as TECurveConfig>::COEFF_A;
    let d = <EdwardsConfig as TECurveConfig>::COEFF_D;
    let y2 = y * y;
    let x2 = (Fq::ONE - y2) * (a - d * y2).inverse()?;
    let mut x = x2.sqrt()?;
    // arkworks' `sqrt` may return either root; canonicalize to the lower half so
    // the stored sign bit selects the branch exactly as circom does.
    if x.into_bigint() > Fq::MODULUS_MINUS_ONE_DIV_TWO {
        x = -x;
    }
    if sign {
        x = -x;
    }
    Some(EdwardsAffine::new_unchecked(x, y))
}

/// babyjub `packPoint`: point → 32 LE bytes (y, with x's "sign" in the top bit).
fn pack(p: &EdwardsAffine) -> [u8; 32] {
    let mut out = [0u8; 32];
    out.copy_from_slice(&p.y.into_bigint().to_bytes_le());
    if p.x.into_bigint() > Fq::MODULUS_MINUS_ONE_DIV_TWO {
        out[31] |= 0x80;
    }
    out
}

/// circom `getBasePoint`: BLAKE-256 find-group-hash, then clear the cofactor (×8).
fn base_point(idx: usize) -> EdwardsProjective {
    for tryidx in 0usize.. {
        let s = format!("PedersenGenerator_{idx:0>32}_{tryidx:0>32}");
        let mut h: [u8; 32] = Blake256::digest(s.as_bytes()).into();
        h[31] &= 0xBF; // circom clears bit 254
        if let Some(p) = unpack(h) {
            return p.into_group().double().double().double(); // ×8
        }
    }
    unreachable!()
}

#[test]
fn matches_circomlibjs_hello_vector() {
    // "Hello" is 40 bits → 10 four-bit windows → a single 200-bit segment,
    // so only base point 0 is exercised.
    let base0 = base_point(0);
    let mut powers = Vec::with_capacity(50);
    let mut cur = base0;
    for _ in 0..50 {
        powers.push(cur);
        for _ in 0..5 {
            cur = cur.double(); // POWER_SHIFT = 5  →  ×32 between windows
        }
    }
    let hasher =
        Pedersen::<EdwardsProjective, Circom, LsbFirst, WholePoint>::from_params(
            Parameters::adopt(vec![powers]),
        );

    let packed = pack(&hasher.hash(b"Hello"));
    let hex: String = packed.iter().map(|b| format!("{b:02x}")).collect();

    assert_eq!(
        hex,
        "0e90d7d613ab8b5ea7f4f8bc537db6bb0fa2e5e97bbac1c1f609ef9e6a35fd8b",
        "should match circomlibjs pedersen.hash(\"Hello\")"
    );
}

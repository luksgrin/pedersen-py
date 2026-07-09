//! circom / iden3-compatible Baby Jubjub Pedersen hash (feature `circom`).
//!
//! Reconstructs circom's real generators (BLAKE-256 find-group-hash) and its
//! `packPoint` output, wired onto the shared skeleton with the [`Circom`]
//! encoding. Byte-compatible with `circomlibjs`' `pedersenHash` (verified in
//! `tests/circom.rs`).

use crate::{Circom, Generators, LsbFirst, OutputEncoding, Parameters, Pedersen};
use ark_babyjubjub::{EdwardsAffine, EdwardsConfig, EdwardsProjective, Fq};
use ark_ec::{twisted_edwards::TECurveConfig, AdditiveGroup, AffineRepr, CurveGroup};
use ark_ff::{BigInt, BigInteger, Field, PrimeField};
use blake_hash::{Blake256, Digest};

/// Baby Jubjub Pedersen hash matching circomlib / iden3.
pub type BabyJubjubCircom = Pedersen<EdwardsProjective, Circom, LsbFirst, Packed>;

/// Build a hasher with capacity for `segments` 200-bit segments.
pub fn hasher(segments: usize) -> BabyJubjubCircom {
    Pedersen::from_params(Parameters::uniform::<Circom, _>(
        &CircomGenerators,
        segments.max(1),
        50,
    ))
}

/// Build a hasher sized for inputs up to `max_bytes` long.
pub fn hasher_for_len(max_bytes: usize) -> BabyJubjubCircom {
    hasher((max_bytes * 8).div_ceil(4).div_ceil(50))
}

/// circom's deterministic generators: one base point per 200-bit segment.
pub struct CircomGenerators;

impl Generators<EdwardsProjective> for CircomGenerators {
    fn bases(&self, num_segments: usize) -> Vec<EdwardsProjective> {
        (0..num_segments).map(base_point).collect()
    }
}

/// babyjub `packPoint`: 32 LE bytes of `v`, with `u`'s half-sign in the top bit.
pub struct Packed;

impl OutputEncoding<EdwardsProjective> for Packed {
    type Output = [u8; 32];
    fn finalize(point: EdwardsProjective) -> [u8; 32] {
        let p = point.into_affine();
        let mut out = [0u8; 32];
        out.copy_from_slice(&p.y.into_bigint().to_bytes_le());
        if p.x.into_bigint() > Fq::MODULUS_MINUS_ONE_DIV_TWO {
            out[31] |= 0x80;
        }
        out
    }
}

/// circom `getBasePoint(idx)`: BLAKE-256 find-group-hash, then clear the cofactor.
pub fn base_point(idx: usize) -> EdwardsProjective {
    for tryidx in 0usize.. {
        let s = format!("PedersenGenerator_{idx:0>32}_{tryidx:0>32}");
        let mut h: [u8; 32] = Blake256::digest(s.as_bytes()).into();
        h[31] &= 0xBF;
        if let Some(p) = unpack(h) {
            return p.into_group().double().double().double(); // ×8
        }
    }
    unreachable!()
}

/// babyjub `unpackPoint`, with the sqrt root canonicalized to the lower half so
/// the stored (half-)sign bit selects the branch exactly as circom does.
fn unpack(mut bytes: [u8; 32]) -> Option<EdwardsAffine> {
    let sign = bytes[31] & 0x80 != 0;
    bytes[31] &= 0x7f;
    let mut limbs = [0u64; 4];
    for (i, limb) in limbs.iter_mut().enumerate() {
        *limb = u64::from_le_bytes(bytes[i * 8..i * 8 + 8].try_into().unwrap());
    }
    let y = Fq::from_bigint(BigInt::new(limbs))?;
    let a = <EdwardsConfig as TECurveConfig>::COEFF_A;
    let d = <EdwardsConfig as TECurveConfig>::COEFF_D;
    let y2 = y * y;
    let x2 = (Fq::ONE - y2) * (a - d * y2).inverse()?;
    let mut x = x2.sqrt()?;
    if x.into_bigint() > Fq::MODULUS_MINUS_ONE_DIV_TWO {
        x = -x;
    }
    if sign {
        x = -x;
    }
    Some(EdwardsAffine::new_unchecked(x, y))
}

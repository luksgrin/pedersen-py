//! circom / iden3-compatible Baby Jubjub Pedersen hash (feature `circom`).
//!
//! Byte-compatible with `circomlibjs`' `pedersenHash` (verified in
//! `tests/circom.rs`). [`BabyJubjubCircom`] is a reusable hasher: it derives each
//! generator once and caches it, growing on demand. The first [`BAKED`] base
//! points are shipped as constants (no BLAKE-256 needed); beyond that they are
//! derived via circom's BLAKE-256 find-group-hash, so arbitrary-length input is
//! supported. A test re-derives the baked table via BLAKE-256 to prevent drift.

use crate::{BitLayout, Circom, Encoding, LsbFirst, Parameters, hash_bits};
use ark_babyjubjub::{EdwardsAffine, EdwardsConfig, EdwardsProjective, Fq};
use ark_ec::{AdditiveGroup, AffineRepr, CurveGroup, twisted_edwards::TECurveConfig};
use ark_ff::{BigInt, BigInteger, Field, PrimeField};
use blake_hash::{Blake256, Digest};
use core::str::FromStr;

/// Four-bit windows, 50 per 200-bit segment.
const CHUNKS_PER_SEGMENT: usize = 50;

/// Precomputed base points `(x, y)` for segments 0..6. These equal circom's
/// BLAKE-256 find-group-hash generators (checked by `baked_bases_match_blake`),
/// and cover inputs up to `6 * 200 = 1200` bits (150 bytes) with no hashing.
const BAKED: [(&str, &str); 6] = [
    (
        "10457101036533406547632367118273992217979173478358440826365724437999023779287",
        "19824078218392094440610104313265183977899662750282163392862422243483260492317",
    ),
    (
        "2671756056509184035029146175565761955751135805354291559563293617232983272177",
        "2663205510731142763556352975002641716101654201788071096152948830924149045094",
    ),
    (
        "5802099305472655231388284418920769829666717045250560929368476121199858275951",
        "5980429700218124965372158798884772646841287887664001482443826541541529227896",
    ),
    (
        "7107336197374528537877327281242680114152313102022415488494307685842428166594",
        "2857869773864086953506483169737724679646433914307247183624878062391496185654",
    ),
    (
        "20265828622013100949498132415626198973119240347465898028410217039057588424236",
        "1160461593266035632937973507065134938065359936056410650153315956301179689506",
    ),
    (
        "1487999857809287756929114517587739322941449154962237464737694709326309567994",
        "14017256862867289575056460215526364897734808720610101650676790868051368668003",
    ),
];

/// A reusable circom Pedersen hasher. Generators are memoized across calls and
/// extended as longer inputs require them.
pub struct BabyJubjubCircom {
    params: Parameters<EdwardsProjective>,
}

impl Default for BabyJubjubCircom {
    fn default() -> Self {
        Self::new()
    }
}

impl BabyJubjubCircom {
    pub fn new() -> Self {
        Self {
            params: Parameters {
                generators: Vec::new(),
                offset: EdwardsProjective::ZERO,
            },
        }
    }

    /// Hash `data`, returning the 32-byte packed point (circomlibjs `packPoint`).
    pub fn hash(&mut self, data: &[u8]) -> [u8; 32] {
        let segments = (data.len() * 8)
            .div_ceil(4)
            .div_ceil(CHUNKS_PER_SEGMENT)
            .max(1);
        self.ensure(segments);
        let point = hash_bits::<EdwardsProjective, Circom>(&self.params, &LsbFirst::expand(data));
        pack(&point.into_affine())
    }

    /// Ensure generators for at least `segments` segments are cached.
    fn ensure(&mut self, segments: usize) {
        while self.params.generators.len() < segments {
            let base = base_point(self.params.generators.len());
            self.params.generators.push(segment_powers(base));
        }
    }
}

/// `[base, base·32, base·32², …]` (`POWER_SHIFT = 5` → ×32 between windows).
fn segment_powers(base: EdwardsProjective) -> Vec<EdwardsProjective> {
    let mut powers = Vec::with_capacity(CHUNKS_PER_SEGMENT);
    let mut cur = base;
    for _ in 0..CHUNKS_PER_SEGMENT {
        powers.push(cur);
        for _ in 0..<Circom as Encoding<EdwardsProjective>>::POWER_SHIFT {
            cur = cur.double();
        }
    }
    powers
}

/// Segment `idx`'s base point: the baked constant if available, else derived.
pub fn base_point(idx: usize) -> EdwardsProjective {
    match BAKED.get(idx) {
        Some(&(x, y)) => {
            EdwardsAffine::new_unchecked(Fq::from_str(x).unwrap(), Fq::from_str(y).unwrap())
                .into_group()
        }
        None => derive_base_point(idx),
    }
}

/// circom `getBasePoint(idx)`: BLAKE-256 find-group-hash, then clear the cofactor.
fn derive_base_point(idx: usize) -> EdwardsProjective {
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

/// babyjub `packPoint`: 32 LE bytes of `v`, with `u`'s half-sign in the top bit.
fn pack(p: &EdwardsAffine) -> [u8; 32] {
    let mut out = [0u8; 32];
    out.copy_from_slice(&p.y.into_bigint().to_bytes_le());
    if p.x.into_bigint() > Fq::MODULUS_MINUS_ONE_DIV_TWO {
        out[31] |= 0x80;
    }
    out
}

/// babyjub `unpackPoint`, sqrt root canonicalized to the lower half so the stored
/// (half-)sign bit selects the branch exactly as circom does.
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The shipped constants must equal the BLAKE-256 derivation (Zcash pattern).
    #[test]
    fn baked_bases_match_blake() {
        for (i, &(x, y)) in BAKED.iter().enumerate() {
            let baked =
                EdwardsAffine::new_unchecked(Fq::from_str(x).unwrap(), Fq::from_str(y).unwrap());
            assert_eq!(baked, derive_base_point(i).into_affine(), "circom base {i}");
        }
    }
}

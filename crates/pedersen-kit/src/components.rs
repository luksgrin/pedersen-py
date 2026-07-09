//! Concrete implementations of the four parameterization axes.
//!
//! Mixing and matching these is what yields each family member.

use crate::{BitLayout, CurveGroup, Encoding, Generators, OutputEncoding};
use ark_ec::twisted_edwards::{Projective as TEProjective, TECurveConfig};
use ark_ff::{BigInteger, PrimeField};
use ark_serialize::CanonicalSerialize;

/* --------------------------------- Encodings -------------------------------- */

/// Plain unsigned binary windows — arkworks `pedersen::CRH` semantics.
///
/// One bit per chunk; a set bit contributes its generator power, a clear bit
/// contributes nothing. Powers are spaced by `2` (single doubling), so a window
/// of bits `b_j` contributes `(Σ_j b_j 2^j)·base`.
pub struct Unsigned;

impl<C: CurveGroup> Encoding<C> for Unsigned {
    const CHUNK_BITS: usize = 1;
    const POWER_SHIFT: usize = 1;

    fn contribute(chunk: &[bool], power: &C) -> C {
        if chunk[0] {
            *power
        } else {
            C::ZERO
        }
    }
}

/// Signed 3-bit chunks — arkworks `bowe_hopwood::CRH` / Zcash Sapling semantics.
///
/// Each chunk `(c0,c1,c2)` encodes the signed digit
/// `enc = (1 − 2·c2)·(1 + c0 + 2·c1) ∈ {−4,…,−1,1,…,4}` and contributes
/// `enc·power`. Powers are spaced by `2^4 = 16` (four doublings). The arithmetic
/// mirrors arkworks line for line.
pub struct BoweHopwood;

impl<C: CurveGroup> Encoding<C> for BoweHopwood {
    const CHUNK_BITS: usize = 3;
    const POWER_SHIFT: usize = 4;

    fn contribute(chunk: &[bool], power: &C) -> C {
        let mut encoded = *power;
        if chunk[0] {
            encoded += power;
        }
        if chunk[1] {
            encoded += power.double();
        }
        if chunk[2] {
            encoded = -encoded;
        }
        encoded
    }
}

/// circomlib / iden3 windowed encoding.
///
/// Each 4-bit chunk is `[m0, m1, m2, sign]` (LSB-first): a magnitude
/// `mag = 1 + m0 + 2·m1 + 4·m2 ∈ [1, 8]` and a sign bit, giving the signed digit
/// `±mag ∈ {−8,…,−1, 1,…,8}`. Consecutive chunks within a segment are spaced by
/// `2^5 = 32` (circom's `exp <<= windowSize + 1`). Mirrors `circomlibjs`'
/// `pedersen_hash` inner loop exactly; trailing bits missing from the last chunk
/// are `false`, matching circom's `o < bits.length` guards.
pub struct Circom;

impl<C: CurveGroup> Encoding<C> for Circom {
    const CHUNK_BITS: usize = 4;
    const POWER_SHIFT: usize = 5;

    fn contribute(chunk: &[bool], power: &C) -> C {
        let mag = 1 + chunk[0] as u64 + 2 * (chunk[1] as u64) + 4 * (chunk[2] as u64);
        let e = *power * C::ScalarField::from(mag);
        if chunk[3] {
            -e
        } else {
            e
        }
    }
}

/* -------------------------------- Bit layouts ------------------------------- */

/// LSB-first within each byte — arkworks `bytes_to_bits` semantics
/// (`bit i = (byte >> i) & 1`).
pub struct LsbFirst;

impl BitLayout for LsbFirst {
    fn expand(input: &[u8]) -> Vec<bool> {
        let mut bits = Vec::with_capacity(input.len() * 8);
        for byte in input {
            for i in 0..8 {
                bits.push((byte >> i) & 1 == 1);
            }
        }
        bits
    }
}

/// MSB-first within each byte (`bit i = (byte >> (7 - i)) & 1`).
pub struct MsbFirst;

impl BitLayout for MsbFirst {
    fn expand(input: &[u8]) -> Vec<bool> {
        let mut bits = Vec::with_capacity(input.len() * 8);
        for byte in input {
            for i in 0..8 {
                bits.push((byte >> (7 - i)) & 1 == 1);
            }
        }
        bits
    }
}

/* -------------------------------- Generators -------------------------------- */

/// A deterministic, reproducible generator source: `base_i = s_i · G`, where `G`
/// is the curve's prime-order generator and `s_i` is a fixed per-index scalar.
///
/// This reuses arkworks' generator + scalar multiplication and is fully
/// reproducible (unlike arkworks' RNG-sampled generators). It is *not* any
/// particular spec's generator set — to match Zcash/circom exactly you would
/// supply their points via [`crate::Parameters`] directly. The domain separator
/// lets distinct instances use disjoint generators.
pub struct Deterministic {
    pub domain: u64,
}

impl Deterministic {
    pub fn new(domain: u64) -> Self {
        Self { domain }
    }
}

impl<C: CurveGroup> Generators<C> for Deterministic {
    fn bases(&self, num_segments: usize) -> Vec<C> {
        let g = C::generator();
        (0..num_segments)
            .map(|i| {
                // Distinct, non-zero scalar per (domain, index).
                let mixed = self
                    .domain
                    .wrapping_add(i as u64)
                    .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                    | 1;
                g * C::ScalarField::from(mixed)
            })
            .collect()
    }
}

/* ------------------------------ Output encodings ---------------------------- */

/// Output the whole point (affine) — arkworks `pedersen::CRH` output.
pub struct WholePoint;

impl<C: CurveGroup> OutputEncoding<C> for WholePoint {
    type Output = C::Affine;
    fn finalize(point: C) -> C::Affine {
        point.into_affine()
    }
}

/// Output the compressed serialization of the point.
pub struct Compressed;

impl<C: CurveGroup> OutputEncoding<C> for Compressed {
    type Output = Vec<u8>;
    fn finalize(point: C) -> Vec<u8> {
        let mut bytes = Vec::new();
        point
            .into_affine()
            .serialize_compressed(&mut bytes)
            .expect("point serialization is infallible");
        bytes
    }
}

/// Output the affine x-coordinate as a base-field element — arkworks
/// `bowe_hopwood::CRH` output. Defined for Twisted Edwards curves, where the
/// x-coordinate is well defined.
pub struct XCoordinate;

impl<P: TECurveConfig> OutputEncoding<TEProjective<P>> for XCoordinate {
    type Output = P::BaseField;
    fn finalize(point: TEProjective<P>) -> P::BaseField {
        point.into_affine().x
    }
}

/// Output the little-endian byte encoding of the x-coordinate (Twisted Edwards).
pub struct XCoordinateBytes;

impl<P: TECurveConfig> OutputEncoding<TEProjective<P>> for XCoordinateBytes
where
    P::BaseField: PrimeField,
{
    type Output = Vec<u8>;
    fn finalize(point: TEProjective<P>) -> Vec<u8> {
        point.into_affine().x.into_bigint().to_bytes_le()
    }
}

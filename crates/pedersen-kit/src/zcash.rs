//! Zcash Sapling Pedersen hash (feature `zcash`).
//!
//! Reconstructs Zcash's real generators (BLAKE2s+URS find-group-hash) and its
//! parity-signed point decompression, wired onto the shared skeleton with the
//! [`BoweHopwood`] encoding and [`XCoordinate`] output. Byte-compatible with
//! `zcash/zcash-test-vectors` (verified in `tests/zcash.rs`).

use crate::{BoweHopwood, Generators, LsbFirst, Parameters, Pedersen, XCoordinate};
use ark_ec::{twisted_edwards::TECurveConfig, AdditiveGroup, AffineRepr};
use ark_ed_on_bls12_381::{EdwardsAffine, EdwardsConfig, EdwardsProjective, Fq};
use ark_ff::{BigInt, BigInteger, Field, PrimeField};

/// The standard personalization for Sapling's windowed Pedersen hash.
pub const ZCASH_PH: [u8; 8] = *b"Zcash_PH";

/// Fixed 64-byte "uniform random string" prepended to every group-hash input.
const URS: &[u8] = b"096b36a5804bfacef1691e173c366a47ff5ba84a44f26ddd7e8d9f79d5b42df0";
/// Chunks per segment (`c` in the Zcash spec).
const SEGMENT_CHUNKS: usize = 63;

/// Jubjub Pedersen hash matching Zcash Sapling (u-coordinate output).
pub type JubjubSapling = Pedersen<EdwardsProjective, BoweHopwood, LsbFirst, XCoordinate>;

/// Build a hasher (for the given personalization) with capacity for `segments`.
pub fn hasher(personalization: [u8; 8], segments: usize) -> JubjubSapling {
    Pedersen::from_params(Parameters::uniform::<BoweHopwood, _>(
        &ZcashGenerators { personalization },
        segments.max(1),
        SEGMENT_CHUNKS,
    ))
}

/// Build a hasher sized for inputs up to `max_bytes` long.
pub fn hasher_for_len(personalization: [u8; 8], max_bytes: usize) -> JubjubSapling {
    hasher(
        personalization,
        (max_bytes * 8).div_ceil(3).div_ceil(SEGMENT_CHUNKS),
    )
}

/// Zcash generators: `I_D_i(D, i) = find_group_hash(D, i2leosp(32, i-1))`.
pub struct ZcashGenerators {
    pub personalization: [u8; 8],
}

impl Generators<EdwardsProjective> for ZcashGenerators {
    fn bases(&self, num_segments: usize) -> Vec<EdwardsProjective> {
        (0..num_segments)
            .map(|s| find_group_hash(&self.personalization, &(s as u32).to_le_bytes()))
            .collect()
    }
}

/// Zcash `find_group_hash`: append an incrementing counter byte until valid.
pub fn find_group_hash(d: &[u8], m: &[u8]) -> EdwardsProjective {
    for i in 0u8..=255 {
        let mut mm = m.to_vec();
        mm.push(i);
        if let Some(p) = group_hash(d, &mm) {
            return p;
        }
    }
    panic!("no valid group hash point found");
}

/// Zcash `group_hash`: BLAKE2s(person=D) over URS‖M, decompress, clear cofactor.
fn group_hash(d: &[u8], m: &[u8]) -> Option<EdwardsProjective> {
    let mut params = blake2s_simd::Params::new();
    params.hash_length(32).personal(d);
    let hash = params.to_state().update(URS).update(m).finalize();
    let bytes: [u8; 32] = hash.as_bytes().try_into().unwrap();

    let p = from_bytes(&bytes)?;
    let q = p.into_group().double().double().double(); // ×8
    if q == EdwardsProjective::ZERO {
        None
    } else {
        Some(q)
    }
}

/// Jubjub `Point.from_bytes`: 32 LE bytes → point, sign from `u`'s parity.
fn from_bytes(buf: &[u8; 32]) -> Option<EdwardsAffine> {
    let u_sign = buf[31] >> 7;
    let mut b = *buf;
    b[31] &= 0x7f;
    let mut limbs = [0u64; 4];
    for (i, limb) in limbs.iter_mut().enumerate() {
        *limb = u64::from_le_bytes(b[i * 8..i * 8 + 8].try_into().unwrap());
    }
    let v = Fq::from_bigint(BigInt::new(limbs))?;
    let a = <EdwardsConfig as TECurveConfig>::COEFF_A;
    let d = <EdwardsConfig as TECurveConfig>::COEFF_D;
    let vv = v * v;
    let u2 = (vv - Fq::ONE) * (vv * d - a).inverse()?;
    let mut u = u2.sqrt()?;
    if (u.into_bigint().is_odd() as u8) != u_sign {
        u = -u;
    }
    Some(EdwardsAffine::new_unchecked(u, v))
}

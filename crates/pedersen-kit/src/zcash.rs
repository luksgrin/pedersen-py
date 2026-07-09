//! Zcash Sapling Pedersen hash (feature `zcash`).
//!
//! Byte-compatible with `zcash/zcash-test-vectors` (verified in `tests/zcash.rs`).
//! [`JubjubSapling`] is a reusable hasher: it derives each generator once and
//! caches it, growing on demand. The first [`BAKED`] generators (for the default
//! [`ZCASH_PH`] personalization) are shipped as constants; beyond that, or for
//! any other personalization, generators are derived via the BLAKE2s+URS
//! find-group-hash, so arbitrary-length input is supported. A test re-derives the
//! baked table via BLAKE2s to prevent drift.

use crate::{hash_bits, BitLayout, BoweHopwood, Encoding, LsbFirst, Parameters};
use ark_ec::{twisted_edwards::TECurveConfig, AdditiveGroup, AffineRepr, CurveGroup};
use ark_ed_on_bls12_381::{EdwardsAffine, EdwardsConfig, EdwardsProjective, Fq};
use ark_ff::{BigInt, BigInteger, Field, PrimeField};
use core::str::FromStr;

/// The standard personalization for Sapling's windowed Pedersen hash.
pub const ZCASH_PH: [u8; 8] = *b"Zcash_PH";

/// Fixed 64-byte "uniform random string" prepended to every group-hash input.
const URS: &[u8] = b"096b36a5804bfacef1691e173c366a47ff5ba84a44f26ddd7e8d9f79d5b42df0";
/// Chunks per segment (`c` in the Zcash spec).
const SEGMENT_CHUNKS: usize = 63;

/// Precomputed generators `(u, v)` for indices 0..6 under [`ZCASH_PH`]. These
/// equal the BLAKE2s find-group-hash generators (checked by
/// `baked_bases_match_blake`) and cover inputs up to `6 * 63 * 3 = 1134` bits.
const BAKED: [(&str, &str); 6] = [
    (
        "52355368488200756720908213129543630848976972731871436319321443845291207170897",
        "18372611905088487385433946659983357101887954355879737496286092836680199584970",
    ),
    (
        "9787319019520772215561425571402619434275350335445140843695488791465664995454",
        "617599303620822769724880923839314378351145790385632133893219494436232173713",
    ),
    (
        "46254521528573726497224586973822974014192468152453531001037375756982829433973",
        "24506313747297525290953778557147418250711256987769181747135349052620150133847",
    ),
    (
        "22718818598176814730279188811725115822910786497974609492339302594899840639692",
        "21482900543196151117444117927157074338652061209517124624989426821710350741737",
    ),
    (
        "27058202516373004425968234366429922161775745886920564416371317087192362222289",
        "33152712010531917481292916450097258839113870850090594303231218126810079660783",
    ),
    (
        "44899967701403962114488563060475643935789150330315799264409868287497276170361",
        "45648747605882624690586248172048386288129541878950585457687885861218308416154",
    ),
];

/// A reusable Zcash Sapling Pedersen hasher. Generators are memoized across calls
/// and extended as longer inputs require them.
pub struct JubjubSapling {
    params: Parameters<EdwardsProjective>,
    personalization: [u8; 8],
}

impl Default for JubjubSapling {
    fn default() -> Self {
        Self::new()
    }
}

impl JubjubSapling {
    /// A hasher for the standard `Zcash_PH` personalization.
    pub fn new() -> Self {
        Self::with_personalization(ZCASH_PH)
    }

    /// A hasher for a custom 8-byte personalization (baked table is bypassed).
    pub fn with_personalization(personalization: [u8; 8]) -> Self {
        Self {
            params: Parameters {
                generators: Vec::new(),
                offset: EdwardsProjective::ZERO,
            },
            personalization,
        }
    }

    /// Hash `data`, returning the u-coordinate.
    pub fn hash(&mut self, data: &[u8]) -> Fq {
        let segments = (data.len() * 8)
            .div_ceil(3)
            .div_ceil(SEGMENT_CHUNKS)
            .max(1);
        self.ensure(segments);
        hash_bits::<EdwardsProjective, BoweHopwood>(&self.params, &LsbFirst::expand(data))
            .into_affine()
            .x
    }

    fn ensure(&mut self, segments: usize) {
        while self.params.generators.len() < segments {
            let base = generator(&self.personalization, self.params.generators.len());
            self.params.generators.push(segment_powers(base));
        }
    }
}

/// `[base, base·16, base·16², …]` (`POWER_SHIFT = 4` → ×16 between chunks).
fn segment_powers(base: EdwardsProjective) -> Vec<EdwardsProjective> {
    let mut powers = Vec::with_capacity(SEGMENT_CHUNKS);
    let mut cur = base;
    for _ in 0..SEGMENT_CHUNKS {
        powers.push(cur);
        for _ in 0..<BoweHopwood as Encoding<EdwardsProjective>>::POWER_SHIFT {
            cur = cur.double();
        }
    }
    powers
}

/// Generator `idx` for `personalization`: the baked constant when it applies,
/// else derived.
fn generator(personalization: &[u8; 8], idx: usize) -> EdwardsProjective {
    if *personalization == ZCASH_PH
        && let Some(&(x, y)) = BAKED.get(idx)
    {
        return EdwardsAffine::new_unchecked(Fq::from_str(x).unwrap(), Fq::from_str(y).unwrap())
            .into_group();
    }
    find_group_hash(personalization, &(idx as u32).to_le_bytes())
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The shipped constants must equal the BLAKE2s derivation (Zcash pattern).
    #[test]
    fn baked_bases_match_blake() {
        for (i, &(x, y)) in BAKED.iter().enumerate() {
            let baked =
                EdwardsAffine::new_unchecked(Fq::from_str(x).unwrap(), Fq::from_str(y).unwrap());
            let derived = find_group_hash(&ZCASH_PH, &(i as u32).to_le_bytes()).into_affine();
            assert_eq!(baked, derived, "zcash base {i}");
        }
    }
}

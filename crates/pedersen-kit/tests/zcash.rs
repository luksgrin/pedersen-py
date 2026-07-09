//! Byte-compatibility with Zcash Sapling's Pedersen hash.
//!
//! Zcash's `encode_chunk`/`encode_segment` are exactly our `BoweHopwood` encoding
//! (signed 3-bit chunks, radix 2⁴), the output is the u-coordinate (`XCoordinate`),
//! and segments hold `c = 63` chunks. The two spec-specific pieces — the BLAKE2s
//! find-group-hash generators and the point decompression — are reconstructed
//! here (TEST-ONLY, via the `blake2s_simd` dev-dep) and driven through the library
//! skeleton. Reference values come from `zcash/zcash-test-vectors` (sapling).
//!
//! arkworks `ed_on_bls12_381` is Zcash's Jubjub curve, so its (x, y) == Zcash's
//! (u, v); the assertion on generator 1 confirms this.

use ark_ec::{twisted_edwards::TECurveConfig, AdditiveGroup, AffineRepr, CurveGroup};
use ark_ed_on_bls12_381::{EdwardsAffine, EdwardsConfig, EdwardsProjective, Fq};
use ark_ff::{BigInt, BigInteger, Field, PrimeField};
use core::str::FromStr;

use pedersen_kit::{BoweHopwood, LsbFirst, Parameters, Pedersen, XCoordinate};

/// The fixed 64-byte "uniform random string" prepended to every group-hash input.
const URS: &[u8] = b"096b36a5804bfacef1691e173c366a47ff5ba84a44f26ddd7e8d9f79d5b42df0";

/// Jubjub `Point.from_bytes`: 32 LE bytes → point, sign taken from u's parity.
fn from_bytes(buf: &[u8; 32]) -> Option<EdwardsAffine> {
    let u_sign = buf[31] >> 7;
    let mut b = *buf;
    b[31] &= 0x7f;

    let mut limbs = [0u64; 4];
    for (i, limb) in limbs.iter_mut().enumerate() {
        *limb = u64::from_le_bytes(b[i * 8..i * 8 + 8].try_into().unwrap());
    }
    let v = Fq::from_bigint(BigInt::new(limbs))?; // None if >= modulus

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

/// Zcash `find_group_hash`: append an incrementing counter byte until valid.
fn find_group_hash(d: &[u8], m: &[u8]) -> EdwardsProjective {
    for i in 0u8..=255 {
        let mut mm = m.to_vec();
        mm.push(i);
        if let Some(p) = group_hash(d, &mm) {
            return p;
        }
    }
    panic!("no valid group hash point found");
}

/// Hash `msg` with the Zcash Sapling configuration and return the u-coordinate.
///
/// Builds one generator per `3·c = 189`-bit segment (`c = 63` chunks), so inputs
/// spanning multiple segments exercise generators for indices 1, 2, … just as
/// `I_D_i(D, i)` does.
fn zcash_hash(msg: &[u8]) -> Fq {
    let d = b"Zcash_PH";
    let n_chunks = (msg.len() * 8).div_ceil(3); // 3-bit chunks
    let n_segments = n_chunks.div_ceil(63).max(1); // c = 63 chunks per segment
    let generators = (0..n_segments)
        .map(|s| {
            // I_D_i(D, i) with i = s + 1  →  find_group_hash(D, i2leosp(32, s)).
            let mut powers = Vec::with_capacity(63);
            let mut cur = find_group_hash(d, &(s as u32).to_le_bytes());
            for _ in 0..63 {
                powers.push(cur);
                for _ in 0..4 {
                    cur = cur.double(); // POWER_SHIFT = 4  →  ×16 between chunks
                }
            }
            powers
        })
        .collect();
    Pedersen::<EdwardsProjective, BoweHopwood, LsbFirst, XCoordinate>::from_params(
        Parameters::adopt(generators),
    )
    .hash(msg)
}

#[test]
fn generator_one_matches_reference() {
    // find_group_hash(D, i2leosp(32, 0)); confirms arkworks Jubjub (x, y) == Zcash (u, v).
    let g = find_group_hash(b"Zcash_PH", &[0, 0, 0, 0]).into_affine();
    assert_eq!(
        g.x,
        Fq::from_str(
            "52355368488200756720908213129543630848976972731871436319321443845291207170897"
        )
        .unwrap(),
        "generator 1 u-coordinate"
    );
    assert_eq!(
        g.y,
        Fq::from_str(
            "18372611905088487385433946659983357101887954355879737496286092836680199584970"
        )
        .unwrap(),
        "generator 1 v-coordinate"
    );
}

#[test]
fn matches_zcash_sapling_vector() {
    // "Hello" = 40 bits → 14 chunks → a single segment (generator index 0 only).
    assert_eq!(
        zcash_hash(b"Hello"),
        Fq::from_str(
            "8754254972755604884333948367738998890971419059392001151429652007230018821080"
        )
        .unwrap(),
        "pedersen_hash(\"Hello\")"
    );
}

#[test]
fn matches_zcash_sapling_multisegment_vector() {
    // bytes 0..64 = 512 bits → 171 chunks → 3 segments, exercising generator
    // indices 0, 1 and 2.
    let msg: Vec<u8> = (0u8..64).collect();
    assert_eq!(
        zcash_hash(&msg),
        Fq::from_str(
            "37515569649653130145499701737487402729855929021425639231448526651152569150619"
        )
        .unwrap(),
        "pedersen_hash(bytes 0..64)"
    );
}

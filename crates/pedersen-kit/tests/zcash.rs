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

#[test]
fn matches_zcash_sapling_vector() {
    let d = b"Zcash_PH";

    // Generator for segment 1 is find_group_hash(D, i2leosp(32, 0)).
    let gen1 = find_group_hash(d, &[0, 0, 0, 0]);
    let gen1a = gen1.into_affine();
    assert_eq!(
        gen1a.x,
        Fq::from_str(
            "52355368488200756720908213129543630848976972731871436319321443845291207170897"
        )
        .unwrap(),
        "generator 1 u-coordinate"
    );
    assert_eq!(
        gen1a.y,
        Fq::from_str(
            "18372611905088487385433946659983357101887954355879737496286092836680199584970"
        )
        .unwrap(),
        "generator 1 v-coordinate"
    );

    // One segment of c = 63 powers (POWER_SHIFT = 4 → ×16 between chunks).
    let mut powers = Vec::with_capacity(63);
    let mut cur = gen1;
    for _ in 0..63 {
        powers.push(cur);
        for _ in 0..4 {
            cur = cur.double();
        }
    }
    let hasher = Pedersen::<EdwardsProjective, BoweHopwood, LsbFirst, XCoordinate>::from_params(
        Parameters::adopt(vec![powers]),
    );

    // "Hello" = 40 bits LSB-first, matching the reference input.
    let out = hasher.hash(b"Hello");
    assert_eq!(
        out,
        Fq::from_str(
            "8754254972755604884333948367738998890971419059392001151429652007230018821080"
        )
        .unwrap(),
        "pedersen_hash u-coordinate"
    );
}

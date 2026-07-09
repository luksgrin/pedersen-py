//! Correctness by construction: the single skeleton reproduces arkworks'
//! `pedersen::CRH` and `bowe_hopwood::CRH` bit-for-bit when handed the same
//! generators. Covered across both curves, both encodings, several window
//! layouts, and an input matrix (empty, edge, near-capacity, seeded-random).

use ark_crypto_primitives::crh::{bowe_hopwood, pedersen, CRHScheme};
use ark_ec::{
    twisted_edwards::{Projective, TECurveConfig},
    CurveGroup,
};
use ark_ff::PrimeField;
use ark_std::{rand::RngCore, test_rng};

use pedersen_kit::instances::{BabyJubjubPedersen, JubjubBoweHopwood};
use pedersen_kit::{BoweHopwood, LsbFirst, Parameters, Pedersen, Unsigned, WholePoint, XCoordinate};

// A handful of window layouts, to prove the engine matches regardless of how the
// generators are split into windows/segments.
macro_rules! window {
    ($name:ident, $ws:expr, $nw:expr) => {
        #[derive(Clone)]
        struct $name;
        impl pedersen::Window for $name {
            const WINDOW_SIZE: usize = $ws;
            const NUM_WINDOWS: usize = $nw;
        }
    };
}
window!(W8x32, 8, 32);
window!(W4x48, 4, 48);
window!(W16x10, 16, 10);
window!(W3x40, 3, 40);

/// Deterministic input matrix sized to fit `cap_bytes`.
fn inputs(cap_bytes: usize) -> Vec<Vec<u8>> {
    let mut rng = test_rng();
    let mut set = vec![
        vec![],
        vec![0x00],
        vec![0xff],
        vec![0x01, 0x80],
        b"pedersen".to_vec(),
        vec![0xAB; cap_bytes],              // exactly at capacity
        vec![0x5A; cap_bytes.saturating_sub(1)], // one below capacity
    ];
    // Seeded-random messages of varied lengths.
    for _ in 0..12 {
        let len = (rng.next_u32() as usize) % (cap_bytes + 1);
        let mut m = vec![0u8; len];
        rng.fill_bytes(&mut m);
        set.push(m);
    }
    set.retain(|m| m.len() <= cap_bytes);
    set
}

/// Unsigned windows must equal `pedersen::CRH` for any curve group.
fn assert_pedersen<C: CurveGroup, W: pedersen::Window>() {
    let mut rng = test_rng();
    let params = pedersen::CRH::<C, W>::setup(&mut rng).unwrap();
    let ours = Pedersen::<C, Unsigned, LsbFirst, WholePoint>::from_params(Parameters::adopt(
        params.generators.clone(),
    ));
    let cap_bytes = W::WINDOW_SIZE * W::NUM_WINDOWS / 8;
    for m in inputs(cap_bytes) {
        let theirs = pedersen::CRH::<C, W>::evaluate(&params, m.clone()).unwrap();
        assert_eq!(theirs, ours.hash(&m), "pedersen mismatch on {} bytes", m.len());
    }
}

/// Signed 3-bit chunks must equal `bowe_hopwood::CRH` for any TE curve.
fn assert_bowe_hopwood<P, W>()
where
    P: TECurveConfig,
    W: pedersen::Window,
    P::BaseField: PrimeField,
{
    let mut rng = test_rng();
    let params = bowe_hopwood::CRH::<P, W>::setup(&mut rng).unwrap();
    let ours = Pedersen::<Projective<P>, BoweHopwood, LsbFirst, XCoordinate>::from_params(
        Parameters::adopt(params.generators.clone()),
    );
    let cap_bytes = W::WINDOW_SIZE * W::NUM_WINDOWS * 3 / 8;
    for m in inputs(cap_bytes) {
        let theirs = bowe_hopwood::CRH::<P, W>::evaluate(&params, m.clone()).unwrap();
        assert_eq!(theirs, ours.hash(&m), "bowe_hopwood mismatch on {} bytes", m.len());
    }
}

type BabyJubjub = ark_babyjubjub::EdwardsProjective;
type BabyJubjubConfig = ark_babyjubjub::EdwardsConfig;
type Jubjub = ark_ed_on_bls12_381::EdwardsProjective;
type JubjubConfig = ark_ed_on_bls12_381::EdwardsConfig;

#[test]
fn unsigned_matches_arkworks_pedersen() {
    // Baby Jubjub (ERC-2494) and Jubjub, across window layouts.
    assert_pedersen::<BabyJubjub, W8x32>();
    assert_pedersen::<BabyJubjub, W4x48>();
    assert_pedersen::<BabyJubjub, W16x10>();
    assert_pedersen::<Jubjub, W8x32>();
    assert_pedersen::<Jubjub, W3x40>();
}

#[test]
fn bowe_hopwood_matches_arkworks() {
    assert_bowe_hopwood::<BabyJubjubConfig, W8x32>();
    assert_bowe_hopwood::<BabyJubjubConfig, W4x48>();
    assert_bowe_hopwood::<JubjubConfig, W8x32>();
    assert_bowe_hopwood::<JubjubConfig, W16x10>();
}

/// The exported instances are reproducible (deterministic generators) and
/// sensitive to input.
#[test]
fn instances_are_deterministic_and_input_sensitive() {
    let a = BabyJubjubPedersen::build(64, 4); // 256-bit capacity
    let b = BabyJubjubPedersen::build(64, 4);
    assert_eq!(a.hash(b"same"), b.hash(b"same"), "generators must be reproducible");
    assert_ne!(a.hash(b"alpha"), a.hash(b"omega"), "distinct inputs should differ");

    let bh = JubjubBoweHopwood::build(16, 40); // 16*40*3 = 1920-bit capacity
    assert_eq!(bh.hash(b"zcash"), bh.hash(b"zcash"));
    assert_ne!(bh.hash(b"zcash"), bh.hash(b"sapling"));
}

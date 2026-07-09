//! Concrete family members, each a parameterization of the one [`Pedersen`]
//! skeleton over an arkworks curve.
//!
//! # In scope (built from the skeleton)
//!
//! * [`BabyJubjubPedersen`] — unsigned windows over the ERC-2494 Baby Jubjub
//!   curve (`ark-babyjubjub`). Same construction as `ark_crypto_primitives`'
//!   `pedersen::CRH`.
//! * [`JubjubBoweHopwood`] — signed 3-bit chunks over Jubjub (`ed_on_bls12_381`),
//!   the Zcash Sapling / `bowe_hopwood::CRH` construction, outputting the
//!   x-coordinate.
//! * [`BabyJubjubBoweHopwood`] — the circom/iden3 *shape* (Bowe–Hopwood over Baby
//!   Jubjub). See the byte-compatibility caveat below.
//!
//! # Dropped (non-parametrizable with existing infrastructure)
//!
//! * **StarkNet Pedersen** — its curve (the STARK curve) is not an arkworks curve
//!   crate, and it hardcodes four generator points + a shift point + a fixed
//!   2-field-element layout. It *fits the formula* ([`crate::Encoding`] =
//!   [`Unsigned`], fixed segmentation `[248,4,248,4]`, an offset, and
//!   [`XCoordinate`] output), but building it would mean shipping non-arkworks
//!   curve arithmetic and spec constants. Per the design rule ("if a piece needs
//!   non-parametrizable parts, drop it") it is intentionally excluded.
//!
//! # Byte-compatibility caveat
//!
//! [`Deterministic`] generators are reproducible but are *not* any spec's
//! generator set, and arkworks' bit/coordinate conventions are used throughout.
//! These instances are therefore sound Pedersen hashes with the right *structure*
//! but are not byte-identical to circomlib/Zcash. Matching a spec exactly means
//! swapping in that spec's generator points (via [`crate::Parameters`]) and its
//! bit/output conventions — a per-spec effort the skeleton makes localized, not
//! free.

use crate::components::{BoweHopwood, Deterministic, LsbFirst, Unsigned, WholePoint, XCoordinate};
use crate::Pedersen;

/// Baby Jubjub group — the ERC-2494 curve (`ark-babyjubjub`).
pub type BabyJubjub = ark_babyjubjub::EdwardsProjective;
/// Jubjub group (Twisted Edwards over the BLS12-381 scalar field).
pub type Jubjub = ark_ed_on_bls12_381::EdwardsProjective;

/// Unsigned-window Pedersen hash over Baby Jubjub (arkworks `pedersen::CRH` shape).
pub type BabyJubjubPedersen = Pedersen<BabyJubjub, Unsigned, LsbFirst, WholePoint>;

/// Bowe–Hopwood / Zcash-Sapling Pedersen hash over Jubjub (x-coordinate output).
pub type JubjubBoweHopwood = Pedersen<Jubjub, BoweHopwood, LsbFirst, XCoordinate>;

/// Bowe–Hopwood Pedersen hash over Baby Jubjub (circom/iden3 *shape*).
pub type BabyJubjubBoweHopwood = Pedersen<BabyJubjub, BoweHopwood, LsbFirst, XCoordinate>;

/// Domain separators so the instances use disjoint generator sets.
mod domain {
    pub const BABYJUBJUB_PEDERSEN: u64 = 0x_BABE_0001;
    pub const JUBJUB_BOWE_HOPWOOD: u64 = 0x_5A17_0002;
    pub const BABYJUBJUB_BOWE_HOPWOOD: u64 = 0x_BABE_0003;
}

impl BabyJubjubPedersen {
    /// `segments` windows of `bits_per_window` bits each (capacity
    /// `segments * bits_per_window` input bits).
    pub fn build(segments: usize, bits_per_window: usize) -> Self {
        Self::uniform(
            &Deterministic::new(domain::BABYJUBJUB_PEDERSEN),
            segments,
            bits_per_window,
        )
    }
}

impl JubjubBoweHopwood {
    /// `segments` segments of `chunks_per_segment` 3-bit chunks each (capacity
    /// `segments * chunks_per_segment * 3` input bits).
    pub fn build(segments: usize, chunks_per_segment: usize) -> Self {
        Self::uniform(
            &Deterministic::new(domain::JUBJUB_BOWE_HOPWOOD),
            segments,
            chunks_per_segment,
        )
    }
}

impl BabyJubjubBoweHopwood {
    /// `segments` segments of `chunks_per_segment` 3-bit chunks each.
    pub fn build(segments: usize, chunks_per_segment: usize) -> Self {
        Self::uniform(
            &Deterministic::new(domain::BABYJUBJUB_BOWE_HOPWOOD),
            segments,
            chunks_per_segment,
        )
    }
}

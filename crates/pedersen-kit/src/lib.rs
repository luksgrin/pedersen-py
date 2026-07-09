//! # pedersen-kit
//!
//! A *single* Pedersen-hash skeleton from which the whole family of widely-known
//! Pedersen hashes is obtained purely by parameterization. Every concrete hash is
//! the same multi-scalar multiplication
//!
//! ```text
//! H(m) = O  +  Σ_k  contribute(chunk_k, power_k)
//! ```
//!
//! where the input is turned into a bit stream, the bits are grouped into
//! fixed-size **chunks**, each chunk is combined with a precomputed **generator
//! power**, the terms are summed, an optional **offset** `O` is added, and the
//! resulting group element is serialized.
//!
//! What distinguishes Zcash Sapling, circomlib, arkworks `pedersen::CRH` and
//! arkworks `bowe_hopwood::CRH` is only *which* choices you plug into that one
//! formula. Those choices are the trait "axes" below:
//!
//! * [`Encoding`]      — chunk size, power spacing, and `contribute` (the digit map)
//! * [`BitLayout`]     — bytes → ordered bit stream
//! * [`Generators`]    — the base points (and offset)
//! * [`OutputEncoding`]— group element → output representation
//!
//! ## Reuse
//!
//! All elliptic-curve and field arithmetic is arkworks (`ark-ec`/`ark-ff`); the
//! curves are arkworks curve crates. The [`Parameters`] layout is intentionally
//! identical to `ark_crypto_primitives`' Pedersen parameters, so this skeleton
//! reproduces arkworks' `pedersen::CRH` and `bowe_hopwood::CRH` **bit-for-bit**
//! when given the same generators (see `tests/parity.rs`).
//!
//! We deliberately do *not* reuse arkworks' `evaluate`/`setup` functions
//! themselves: their generators are sampled from an RNG (non-reproducible) and
//! their input/output conventions are fixed — i.e. the parts that must be
//! *parametrizable* here are hard-coded there. Per the design rule, we drop those
//! specific pieces and keep the arithmetic.

use core::marker::PhantomData;

pub use ark_ec::CurveGroup;

pub mod components;
pub mod instances;

pub use components::*;

/// Precomputed parameters of a configured Pedersen hash.
///
/// `generators[i]` holds the successive powers for segment `i`:
/// `[base_i, base_i·R, base_i·R², …]` with `R = 2^{Encoding::POWER_SHIFT}`.
/// This is exactly the shape of `ark_crypto_primitives::crh::pedersen::Parameters`.
#[derive(Clone, Debug)]
pub struct Parameters<C: CurveGroup> {
    pub generators: Vec<Vec<C>>,
    pub offset: C,
}

impl<C: CurveGroup> Parameters<C> {
    /// Build uniform parameters: `segments` segments, each with `chunks_per_segment`
    /// generator powers, derived from a [`Generators`] source.
    pub fn uniform<E, G>(gens: &G, segments: usize, chunks_per_segment: usize) -> Self
    where
        E: Encoding<C>,
        G: Generators<C>,
    {
        Self::with_segment_sizes::<E, G>(gens, &vec![chunks_per_segment; segments])
    }

    /// Build parameters with explicit per-segment sizes (e.g. StarkNet-style
    /// fixed layouts such as `[248, 4, 248, 4]`).
    pub fn with_segment_sizes<E, G>(gens: &G, sizes: &[usize]) -> Self
    where
        E: Encoding<C>,
        G: Generators<C>,
    {
        let bases = gens.bases(sizes.len());
        let generators = bases
            .into_iter()
            .zip(sizes)
            .map(|(base, &len)| {
                let mut powers = Vec::with_capacity(len);
                let mut cur = base;
                for _ in 0..len {
                    powers.push(cur);
                    for _ in 0..E::POWER_SHIFT {
                        cur = cur.double();
                    }
                }
                powers
            })
            .collect();
        Parameters {
            generators,
            offset: gens.offset(),
        }
    }

    /// Adopt an externally-built generator layout (e.g. arkworks' own
    /// `pedersen`/`bowe_hopwood` parameters), with a zero offset.
    pub fn adopt(generators: Vec<Vec<C>>) -> Self {
        Parameters {
            generators,
            offset: C::ZERO,
        }
    }

    /// Total number of chunks (generator powers) this configuration can absorb.
    pub fn capacity_chunks(&self) -> usize {
        self.generators.iter().map(Vec::len).sum()
    }
}

/// The one and only engine: `offset + Σ_k contribute(chunk_k, power_k)`.
///
/// Bits are grouped into chunks of `E::CHUNK_BITS`; chunk `k` uses the `k`-th
/// generator power (segments flattened in order). Trailing bits missing from the
/// last chunk are treated as `false`, matching arkworks' padding of the final
/// chunk. Chunks beyond the input are simply not processed.
pub fn hash_bits<C, E>(params: &Parameters<C>, bits: &[bool]) -> C
where
    C: CurveGroup,
    E: Encoding<C>,
{
    assert!(E::CHUNK_BITS >= 1 && E::CHUNK_BITS <= 8);
    let powers: Vec<C> = params.generators.iter().flatten().copied().collect();
    let num_chunks = bits.len().div_ceil(E::CHUNK_BITS);
    assert!(
        num_chunks <= powers.len(),
        "input of {} bits needs {} chunks but only {} generator powers are available",
        bits.len(),
        num_chunks,
        powers.len(),
    );

    let mut acc = params.offset;
    let mut buf = [false; 8];
    for (k, power) in powers.iter().take(num_chunks).enumerate() {
        for (t, slot) in buf.iter_mut().enumerate().take(E::CHUNK_BITS) {
            let idx = k * E::CHUNK_BITS + t;
            *slot = idx < bits.len() && bits[idx];
        }
        acc += E::contribute(&buf[..E::CHUNK_BITS], power);
    }
    acc
}

/// A fully configured Pedersen hash function — one type per family member.
pub struct Pedersen<C: CurveGroup, E, B, O> {
    params: Parameters<C>,
    _marker: PhantomData<(E, B, O)>,
}

impl<C, E, B, O> Pedersen<C, E, B, O>
where
    C: CurveGroup,
    E: Encoding<C>,
    B: BitLayout,
    O: OutputEncoding<C>,
{
    /// Wrap explicit parameters (also the path used to adopt arkworks generators).
    pub fn from_params(params: Parameters<C>) -> Self {
        Self {
            params,
            _marker: PhantomData,
        }
    }

    /// Configure with a uniform generator layout derived from a [`Generators`] source.
    pub fn uniform<G: Generators<C>>(gens: &G, segments: usize, chunks_per_segment: usize) -> Self {
        Self::from_params(Parameters::uniform::<E, G>(gens, segments, chunks_per_segment))
    }

    pub fn params(&self) -> &Parameters<C> {
        &self.params
    }

    /// Hash raw bytes.
    pub fn hash(&self, input: &[u8]) -> O::Output {
        O::finalize(hash_bits::<C, E>(&self.params, &B::expand(input)))
    }

    /// Hash a pre-expanded bit stream (bypasses [`BitLayout`]).
    pub fn hash_bits(&self, bits: &[bool]) -> O::Output {
        O::finalize(hash_bits::<C, E>(&self.params, bits))
    }
}

/// Axis — how a fixed-size chunk of bits becomes a curve contribution.
///
/// `CHUNK_BITS` bits are consumed per generator power; consecutive powers within
/// a segment are spaced by `R = 2^POWER_SHIFT` (baked into [`Parameters`]).
pub trait Encoding<C: CurveGroup> {
    const CHUNK_BITS: usize;
    const POWER_SHIFT: usize;
    fn contribute(chunk: &[bool], power: &C) -> C;
}

/// Axis — bytes to an ordered bit stream (endianness / bit order).
pub trait BitLayout {
    fn expand(input: &[u8]) -> Vec<bool>;
}

/// Axis — the base points, and an optional constant offset (StarkNet shift /
/// commitment blinding base).
pub trait Generators<C: CurveGroup> {
    fn bases(&self, num_segments: usize) -> Vec<C>;
    fn offset(&self) -> C {
        C::ZERO
    }
}

/// Axis — the final group element to an output representation.
pub trait OutputEncoding<C: CurveGroup> {
    type Output;
    fn finalize(point: C) -> Self::Output;
}

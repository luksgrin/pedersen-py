//! Python bindings for `pedersen-kit`.
//!
//! Exposes concrete, monomorphized members of the Pedersen hash family (the core
//! crate's generic skeleton can't cross the FFI boundary directly, so each
//! exported class is one fully-configured instance). Each class is constructed
//! with its window layout and hashes bytes, returning the serialized output.

use ark_serialize::CanonicalSerialize;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

/// Serialize any arkworks value (curve point or field element) to compressed bytes.
fn to_bytes<T: CanonicalSerialize>(value: &T) -> Vec<u8> {
    let mut bytes = Vec::new();
    value
        .serialize_compressed(&mut bytes)
        .expect("serialization is infallible");
    bytes
}

/// Unsigned-window Pedersen hash over the ERC-2494 Baby Jubjub curve
/// (arkworks `pedersen::CRH` construction). Output is the compressed curve point.
#[pyclass]
struct BabyJubjubPedersen {
    inner: pedersen_kit::instances::BabyJubjubPedersen,
}

#[pymethods]
impl BabyJubjubPedersen {
    /// `segments` windows of `bits_per_window` bits each
    /// (capacity `segments * bits_per_window` input bits).
    #[new]
    fn new(segments: usize, bits_per_window: usize) -> Self {
        Self {
            inner: pedersen_kit::instances::BabyJubjubPedersen::build(segments, bits_per_window),
        }
    }

    /// Hash `data`, returning the compressed point as bytes.
    fn hash<'py>(&self, py: Python<'py>, data: &[u8]) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &to_bytes(&self.inner.hash(data)))
    }
}

/// Bowe–Hopwood / Zcash-Sapling Pedersen hash over Jubjub
/// (arkworks `bowe_hopwood::CRH` construction). Output is the x-coordinate.
#[pyclass]
struct JubjubBoweHopwood {
    inner: pedersen_kit::instances::JubjubBoweHopwood,
}

#[pymethods]
impl JubjubBoweHopwood {
    /// `segments` segments of `chunks_per_segment` 3-bit chunks each
    /// (capacity `segments * chunks_per_segment * 3` input bits).
    #[new]
    fn new(segments: usize, chunks_per_segment: usize) -> Self {
        Self {
            inner: pedersen_kit::instances::JubjubBoweHopwood::build(segments, chunks_per_segment),
        }
    }

    /// Hash `data`, returning the x-coordinate as bytes.
    fn hash<'py>(&self, py: Python<'py>, data: &[u8]) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &to_bytes(&self.inner.hash(data)))
    }
}

#[pymodule]
fn pedersenpy(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<BabyJubjubPedersen>()?;
    m.add_class::<JubjubBoweHopwood>()?;
    Ok(())
}

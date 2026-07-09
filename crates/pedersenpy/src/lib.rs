//! Python bindings for `pedersen-kit`.
//!
//! Exposes concrete, monomorphized members of the Pedersen hash family (the core
//! crate's generic skeleton can't cross the FFI boundary directly, so each
//! exported class is one fully-configured instance). Every class hashes bytes and
//! returns the serialized output. `CircomPedersen` and `ZcashPedersen` are
//! byte-compatible with circomlibjs and Zcash Sapling respectively.

use ark_serialize::CanonicalSerialize;
use pyo3::exceptions::PyValueError;
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

/// circom / iden3-compatible Baby Jubjub Pedersen hash (`circomlibjs pedersenHash`).
///
/// `hash(data)` returns the 32-byte packed point, byte-identical to circomlibjs.
/// Reusable: generators are derived once and cached across calls, so reuse the
/// same instance for repeated hashing.
#[pyclass]
struct CircomPedersen {
    inner: pedersen_kit::circom::BabyJubjubCircom,
}

#[pymethods]
impl CircomPedersen {
    #[new]
    fn new() -> Self {
        Self {
            inner: pedersen_kit::circom::BabyJubjubCircom::new(),
        }
    }

    /// Hash `data`, returning the 32-byte packed point (circomlibjs `packPoint`).
    fn hash<'py>(&mut self, py: Python<'py>, data: &[u8]) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.inner.hash(data))
    }
}

/// Zcash Sapling Pedersen hash over Jubjub.
///
/// `hash(data)` returns the 32-byte little-endian u-coordinate. The optional
/// 8-byte `personalization` defaults to `b"Zcash_PH"`. Reusable: generators are
/// derived once and cached across calls.
#[pyclass]
struct ZcashPedersen {
    inner: pedersen_kit::zcash::JubjubSapling,
}

#[pymethods]
impl ZcashPedersen {
    #[new]
    #[pyo3(signature = (personalization = None))]
    fn new(personalization: Option<&[u8]>) -> PyResult<Self> {
        let inner = match personalization {
            None => pedersen_kit::zcash::JubjubSapling::new(),
            Some(bytes) => {
                let p: [u8; 8] = bytes.try_into().map_err(|_| {
                    PyValueError::new_err("personalization must be exactly 8 bytes")
                })?;
                pedersen_kit::zcash::JubjubSapling::with_personalization(p)
            }
        };
        Ok(ZcashPedersen { inner })
    }

    /// Hash `data`, returning the 32-byte little-endian u-coordinate.
    fn hash<'py>(&mut self, py: Python<'py>, data: &[u8]) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &to_bytes(&self.inner.hash(data)))
    }
}

#[pymodule]
fn _pedersenpy(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<BabyJubjubPedersen>()?;
    m.add_class::<JubjubBoweHopwood>()?;
    m.add_class::<CircomPedersen>()?;
    m.add_class::<ZcashPedersen>()?;
    Ok(())
}

use pyo3::prelude::*;
use pyo3::types::PyDict;

use num_traits::Num;
use num_bigint::{BigInt, BigUint};

use ff_ce::{PrimeField, hex};
use babyjubjub_rs::{Fr, Point as Babyjubjub_point, PointProjective as Babyjubjub_point_projective};

/*´:°•.°+.*•´.*:˚.°*.˚•´.°:°•.°•.*•´.*:˚.°*.˚•´.°:°•.°+.*•´.*:*/
/*                       Python Module                        */
/*.•°:°.´+˚.*°.˚:*.´•*.+°.•°:´*.´•*.•°.•°:°.´:•˚°.*°.˚:*.´+°.•*/
#[pymodule]
fn pedersenpy(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyBabyjubjubPoint>()?;
    m.add_class::<PyBabyjubjubPointProjective>()?;
    Ok(())
}

/*´:°•.°+.*•´.*:˚.°*.˚•´.°:°•.°•.*•´.*:˚.°*.˚•´.°:°•.°+.*•´.*:*/
/*                       Python Classes                       */
/*.•°:°.´+˚.*°.˚:*.´•*.+°.•°:´*.´•*.•°.•°:°.´:•˚°.*°.˚:*.´+°.•*/
#[pyclass]
struct PyBabyjubjubPoint {
    inner: Babyjubjub_point,
}

#[pyclass]
struct PyBabyjubjubPointProjective {
    inner: Babyjubjub_point_projective,
}

/*´:°•.°+.*•´.*:˚.°*.˚•´.°:°•.°•.*•´.*:˚.°*.˚•´.°:°•.°+.*•´.*:*/
/*                    Python Class Methods                    */
/*.•°:°.´+˚.*°.˚:*.´•*.+°.•°:´*.´•*.•°.•°:°.´:•˚°.*°.˚:*.´+°.•*/
#[pymethods]
impl PyBabyjubjubPoint {

    #[new]
    fn new(py: Python, x: PyObject, y: PyObject) -> PyResult<Self> {
        let x_str = pyobject_to_rust_string(py, x)?;
        let y_str = pyobject_to_rust_string(py, y)?;
        Ok(Self {
            inner: Babyjubjub_point {
                x: Fr::from_str(&x_str).unwrap(),
                y: Fr::from_str(&y_str).unwrap(),
            }
        })
    }

    #[getter]
    fn x(&self) -> BigUint {
        let binding = self.inner.x.into_repr();
        let u64_slice = binding.as_ref();
        field_element_to_biguint(u64_slice)
    }

    #[getter]
    fn y(&self) -> BigUint {
        let binding = self.inner.y.into_repr();
        let u64_slice = binding.as_ref();
        field_element_to_biguint(u64_slice)
    }

    #[getter]
    fn __dict__(&self, py: Python) -> PyObject {
        let dict = PyDict::new(py);
        let _ = dict.set_item("x", &self.x());
        let _ = dict.set_item("y", &self.y());
        dict.into()
    }

    #[pyo3(name = "__str__")]
    fn str_(&self) -> String {
        // Return a user-friendly string representation
        format!(
            "PyBabyjubjubPoint(x={}, y={})",
            biguint_to_hex_string(self.x()),
            biguint_to_hex_string(self.y())
        )
    }

    #[pyo3(name = "__repr__")]
    fn repr_(&self) -> String {
        self.str_()
    }

    #[pyo3(name = "__mul__")]
    fn mul_(&self, py: Python, n: PyObject) -> PyResult<Self> {
        self.mul_scalar(py, n)
    }

    #[pyo3(name = "__eq__")]
    fn eq_(&self, other: &PyBabyjubjubPoint) -> bool {
        self.inner.equals(other.inner.clone())
    }

    fn compress(&self) -> Vec<u8> {
        self.inner.compress().to_vec()
    }

    fn compress_hex(&self) -> String {
        format!(
            "0x{}",
            hex::encode(self.inner.compress())
        )
    }

    fn mul_scalar(&self, py: Python, n: PyObject) -> PyResult<Self> {
        // Convert the Python object to a BigInt
        let n_bigint = pyobject_to_bigint(py, n)?;
        let result = self.inner.mul_scalar(&n_bigint);
        Ok(PyBabyjubjubPoint { inner: result })
    }

    fn to_projective(&self) -> PyBabyjubjubPointProjective {
        PyBabyjubjubPointProjective {
            inner: self.inner.projective(),
        }
    }
    
}

#[pymethods]
impl PyBabyjubjubPointProjective {

    #[new]
    fn new(py: Python, x: PyObject, y: PyObject, z: PyObject) -> PyResult<Self> {
        let x_str = pyobject_to_rust_string(py, x)?;
        let y_str = pyobject_to_rust_string(py, y)?;
        let z_str = pyobject_to_rust_string(py, z)?;
        Ok(Self {
            inner: Babyjubjub_point_projective {
                x: Fr::from_str(&x_str).unwrap(),
                y: Fr::from_str(&y_str).unwrap(),
                z: Fr::from_str(&z_str).unwrap(),
            }
        })
    }

    #[getter]
    fn x(&self) -> BigUint {
        let binding = self.inner.x.into_repr();
        let u64_slice = binding.as_ref();
        field_element_to_biguint(u64_slice)
    }

    #[getter]
    fn y(&self) -> BigUint {
        let binding = self.inner.y.into_repr();
        let u64_slice = binding.as_ref();
        field_element_to_biguint(u64_slice)
    }

    #[getter]
    fn z(&self) -> BigUint {
        let binding = self.inner.z.into_repr();
        let u64_slice = binding.as_ref();
        field_element_to_biguint(u64_slice)
    }

    #[pyo3(name = "__str__")]
    fn str_(&self) -> String {
        format!(
            "PyBabyjubjubPointProjective(x={}, y={}, z={})",
            biguint_to_hex_string(self.x()),
            biguint_to_hex_string(self.y()),
            biguint_to_hex_string(self.z())
        )
    }

    #[pyo3(name = "__repr__")]
    fn repr_(&self) -> String {
        self.str_()
    }

    #[pyo3(name = "__mul__")]
    fn mul_(&self, py: Python, n: PyObject) -> PyResult<Self> {
        self.mul_scalar(py, n)
    }

    #[pyo3(name = "__eq__")]
    fn eq_(&self, other: &PyBabyjubjubPointProjective) -> bool {
        self.inner.affine().equals(other.inner.affine())
    }

    fn mul_scalar(&self, py: Python, n: PyObject) -> PyResult<Self> {
        let n_bigint = pyobject_to_bigint(py, n)?;
        let affine = self.inner.affine();
        let result = affine.mul_scalar(&n_bigint);
        Ok(PyBabyjubjubPointProjective { inner: result.projective() })
    }

    fn to_affine(&self) -> PyBabyjubjubPoint {
        PyBabyjubjubPoint {
            inner: self.inner.affine(),
        }
    }
}


/*´:°•.°+.*•´.*:˚.°*.˚•´.°:°•.°•.*•´.*:˚.°*.˚•´.°:°•.°+.*•´.*:*/
/*                      Helper Functions                      */
/*.•°:°.´+˚.*°.˚:*.´•*.+°.•°:´*.´•*.•°.•°:°.´:•˚°.*°.˚:*.´+°.•*/

fn pyobject_to_rust_string(py: Python, obj: PyObject) -> Result<String, PyErr> {
    // Try to extract as integer
    if let Ok(val) = obj.extract::<usize>(py) {
        Ok(val.to_string())
    // Try to extract as string
    } else if let Ok(val) = obj.extract::<&str>(py) {
        if val.starts_with("0x") || val.starts_with("0X") {
            // Handle hex string (e.g., parse or return as-is)
            Ok(parse_hex_to_bigint(val).unwrap().to_string())
        } else {
            // Handle decimal string
            Ok(val.to_string())
        }
    } else if let Ok(val) = obj.extract::<&[u8]>(py) {
        Ok(BigInt::from_bytes_be(num_bigint::Sign::Plus, val).to_string())
    } else {
        Err(pyo3::exceptions::PyTypeError::new_err(
            "Expected an int or string-like Python object",
        ))
    }
}

fn pyobject_to_bigint(py: Python, obj: PyObject) -> PyResult<BigInt> {
    if let Ok(val) = obj.extract::<usize>(py) {
        return Ok(BigInt::from(val as i64));
    } else if let Ok(val) = obj.extract::<i64>(py) {
        return Ok(BigInt::from(val));
    } else if let Ok(val) = obj.extract::<&str>(py) {
        // Try decimal first, then hex
        if let Some(bi) = BigInt::parse_bytes(val.as_bytes(), 10) {
            return Ok(bi);
        }
        if let Some(hex) = val.strip_prefix("0x").or_else(|| val.strip_prefix("0X")) {
            if let Some(bi) = BigInt::parse_bytes(hex.as_bytes(), 16) {
                return Ok(bi);
            }
        }
    } else if let Ok(val) = obj.extract::<&[u8]>(py) {
        return Ok(BigInt::from_bytes_be(num_bigint::Sign::Plus, val));
    }
    Err(pyo3::exceptions::PyTypeError::new_err(
        "Expected an int, string, or bytes-like Python object for scalar",
    ))
}

fn parse_hex_to_bigint(hex_str: &str) -> Result<BigInt, String> {
    // Remove "0x" or "0X" prefix if present
    let hex = hex_str.strip_prefix("0x").or_else(|| hex_str.strip_prefix("0X")).unwrap_or(hex_str);
    BigInt::from_str_radix(hex, 16)
        .map_err(|e| format!("Failed to parse hex: {}", e))
}

fn field_element_to_biguint(repr: &[u64]) -> BigUint {
    let mut bytes = Vec::with_capacity(repr.len() * 8);
    // Reverse for little-endian to big-endian conversion
    for &word in repr.iter().rev() {
        bytes.extend_from_slice(&word.to_be_bytes());
    }
    BigUint::from_bytes_be(&bytes)
}

fn biguint_to_hex_string(biguint: BigUint) -> String {
    let bytes = biguint.to_bytes_be();
    let hex_str = format!("0x{}", hex::encode(bytes));
    hex_str
}
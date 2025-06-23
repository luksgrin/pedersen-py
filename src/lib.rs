use pyo3::prelude::*;
use num_traits::Num;
use num_bigint::BigInt;


use ff_ce::PrimeField;
use babyjubjub_rs::{Fr, Point as Babyjubjub_point};

/*´:°•.°+.*•´.*:˚.°*.˚•´.°:°•.°•.*•´.*:˚.°*.˚•´.°:°•.°+.*•´.*:*/
/*                       Python Module                        */
/*.•°:°.´+˚.*°.˚:*.´•*.+°.•°:´*.´•*.•°.•°:°.´:•˚°.*°.˚:*.´+°.•*/
#[pymodule]
fn pedersenpy(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyBabyjubjubPoint>()?;
    Ok(())
}

/*´:°•.°+.*•´.*:˚.°*.˚•´.°:°•.°•.*•´.*:˚.°*.˚•´.°:°•.°+.*•´.*:*/
/*                       Python Classes                       */
/*.•°:°.´+˚.*°.˚:*.´•*.+°.•°:´*.´•*.•°.•°:°.´:•˚°.*°.˚:*.´+°.•*/

#[pyclass]
struct PyBabyjubjubPoint {
    inner: Babyjubjub_point,
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

    pub fn x(&self) -> usize {
        parse_hex_to_usize(&self.inner.x.into_repr().to_string()).unwrap()
    }

    pub fn y(&self) -> usize {
        parse_hex_to_usize(&self.inner.y.into_repr().to_string()).unwrap()
    }
}

/*´:°•.°+.*•´.*:˚.°*.˚•´.°:°•.°•.*•´.*:˚.°*.˚•´.°:°•.°+.*•´.*:*/
/*                      Helper Functions                      */
/*.•°:°.´+˚.*°.˚:*.´•*.+°.•°:´*.´•*.•°.•°:°.´:•˚°.*°.˚:*.´+°.•*/

fn pyobject_to_rust_string(py: Python, obj: PyObject) -> Result<String, PyErr> {
    // Try to extract as integer
    if let Ok(val) = obj.extract::<i64>(py) {
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
    } else {
        Err(pyo3::exceptions::PyTypeError::new_err(
            "Expected an int or string-like Python object",
        ))
    }
}

fn parse_hex_to_bigint(hex_str: &str) -> Result<BigInt, String> {
    // Remove "0x" or "0X" prefix if present
    let hex = hex_str.strip_prefix("0x").or_else(|| hex_str.strip_prefix("0X")).unwrap_or(hex_str);
    BigInt::from_str_radix(hex, 16)
        .map_err(|e| format!("Failed to parse hex: {}", e))
}

fn parse_hex_to_usize(hex_str: &str) -> Result<usize, std::num::ParseIntError> {
    // Remove "0x" or "0X" prefix if present
    let hex = hex_str.strip_prefix("0x").or_else(|| hex_str.strip_prefix("0X")).unwrap_or(hex_str);
    usize::from_str_radix(hex, 16)
}
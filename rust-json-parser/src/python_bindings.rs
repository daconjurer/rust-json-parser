use crate::parse_json as parse;
use crate::{JsonError, JsonValue};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

/// Utility function to convert a JsonValue instance (value) into a PyAny instance
fn json_value_to_py<'py>(value: JsonValue, py: Python<'py>) -> Result<Bound<'py, PyAny>, PyErr> {
    match value {
        JsonValue::Null => Ok(py.None().into_bound(py)),
        JsonValue::Boolean(b) => Ok(b.into_pyobject(py)?.to_owned().into_any()),
        JsonValue::Number(n) => Ok(n.into_pyobject(py)?.to_owned().into_any()),
        JsonValue::String(s) => Ok(s.into_pyobject(py)?.to_owned().into_any()),
        JsonValue::Array(arr) => {
            let items: Vec<_> = arr
                .into_iter()
                .map(|v| json_value_to_py(v, py))
                .collect::<Result<Vec<_>, _>>()?;
            let list = PyList::new(py, items)?;
            Ok(list.to_owned().into_any())
        }
        JsonValue::Object(obj) => {
            let dict = PyDict::new(py);
            for (k, v) in obj {
                dict.set_item(k, json_value_to_py(v, py)?)?;
            }
            Ok(dict.into_any())
        }
    }
}

impl<'py> IntoPyObject<'py> for JsonValue {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        json_value_to_py(self, py)
    }
}

impl From<JsonError> for PyErr {
    fn from(err: JsonError) -> PyErr {
        match err {
            JsonError::UnexpectedToken {
                expected,
                found,
                position,
            } => PyValueError::new_err(format!(
                "Unexpected token at position {}: expected {}, found {}",
                position, expected, found
            )),
            JsonError::UnexpectedEndOfInput { expected, position } => {
                PyValueError::new_err(format!(
                    "Unexpected end of input at position {}: expected {}",
                    position, expected
                ))
            }
            JsonError::InvalidNumber { value, position } => PyValueError::new_err(format!(
                "Invalid numeric value at position {}: {}",
                position, value
            )),
            JsonError::InvalidEscape { char, position } => PyValueError::new_err(format!(
                "Invalid escape sequence at position {}: {}",
                position, char
            )),
            JsonError::InvalidUnicode { sequence, position } => PyValueError::new_err(format!(
                "Invalid unicode sequence at position {}: {}",
                position, sequence
            )),
        }
    }
}

#[pyfunction]
fn parse_json<'py>(py: Python<'py>, input: &str) -> PyResult<Bound<'py, PyAny>> {
    let result = parse(input)?;
    result.into_pyobject(py)
}

#[pymodule]
fn _rust_json_parser(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse_json, m)?)?;
    Ok(())
}

use crate::parse_json as parse;
use crate::parse_json_file as parse_file;
use crate::{JsonError, JsonValue};
use pyo3::exceptions::{PyIOError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::collections::HashMap;

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

/// Utility function to convert a PyAny instance (value) into a JsonValue instance
fn py_to_json_value(obj: &Bound<PyAny>) -> PyResult<JsonValue> {
    if obj.is_none() {
        return Ok(JsonValue::Null);
    }
    if let Ok(b) = obj.extract::<bool>() {
        return Ok(JsonValue::Boolean(b));
    }
    if let Ok(n) = obj.extract::<f64>() {
        return Ok(JsonValue::Number(n));
    }
    if let Ok(s) = obj.extract::<String>() {
        return Ok(JsonValue::String(s));
    }
    if let Ok(list) = obj.cast::<PyList>() {
        let arr: Vec<_> = list
            .into_iter()
            .map(|v| py_to_json_value(&v))
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(JsonValue::Array(arr));
    }
    if let Ok(dict) = obj.cast::<PyDict>() {
        let mut object = HashMap::new();
        for (k, v) in dict.iter() {
            let key: String = k.extract()?;
            object.insert(key, py_to_json_value(&v)?);
        }
        return Ok(JsonValue::Object(object));
    }

    Err(PyTypeError::new_err(format!("{:?}", obj)))
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
            JsonError::Io { message } => PyIOError::new_err(message),
        }
    }
}

/// Parse a JSON string and return the corresponding Python object.
///
/// Args:
///     input: A string containing valid JSON.
///
/// Returns:
///     The parsed JSON as a Python object (dict, list, str, float, bool, or None).
///
/// Raises:
///     ValueError: If the input is not valid JSON.
///
/// Examples:
///     >>> parse_json('{"name": "Alice", "age": 30}')
///     {'name': 'Alice', 'age': 30.0}
///
///     >>> parse_json('[1, 2, 3]')
///     [1.0, 2.0, 3.0]
///
///     >>> parse_json('"hello"')
///     'hello'
///
///     >>> parse_json('null')
#[pyfunction]
fn parse_json<'py>(py: Python<'py>, input: &str) -> PyResult<Bound<'py, PyAny>> {
    let result = parse(input)?;
    result.into_pyobject(py)
}

/// Parse a JSON file and return the corresponding Python object.
///
/// Args:
///     path: Path to a file containing valid JSON.
///
/// Returns:
///     The parsed JSON as a Python object (dict, list, str, float, bool, or None).
///
/// Raises:
///     ValueError: If the file contents are not valid JSON.
///     OSError: If the file cannot be read.
///
/// Examples:
///     >>> parse_json_file("config.json")
///     {'key': 'value'}
///
///     >>> parse_json_file("data/users.json")
///     [{'name': 'Alice'}, {'name': 'Bob'}]
#[pyfunction]
fn parse_json_file<'py>(py: Python<'py>, path: &str) -> PyResult<Bound<'py, PyAny>> {
    let result = parse_file(path)?;
    result.into_pyobject(py)
}

/// Serialize a Python object to a JSON string.
///
/// Args:
///     obj: A Python object to serialize (dict, list, str, float, int, bool, or None).
///     indent: Optional number of spaces for pretty-printing. If None, output is compact.
///
/// Returns:
///     A JSON string representation of the object.
///
/// Raises:
///     TypeError: If the object contains types that cannot be serialized to JSON.
///
/// Examples:
///     >>> dumps({"name": "Alice", "age": 30})
///     '{"name": "Alice", "age": 30}'
///
///     >>> dumps([1, 2, 3])
///     '[1, 2, 3]'
///
///     >>> print(dumps({"key": "value"}, indent=2))
///     {
///       "key": "value"
///     }
///
///     >>> dumps(None)
///     'null'
#[pyfunction]
#[pyo3(signature = (obj, indent=None))]
fn dumps(obj: &Bound<PyAny>, indent: Option<usize>) -> PyResult<String> {
    match indent {
        Some(indent) => Ok(py_to_json_value(obj)?.pretty_print(indent)),
        None => Ok(py_to_json_value(obj)?.to_string()),
    }
}

#[pymodule]
fn _rust_json_parser(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse_json, m)?)?;
    m.add_function(wrap_pyfunction!(parse_json_file, m)?)?;
    m.add_function(wrap_pyfunction!(dumps, m)?)?;
    Ok(())
}

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyString, PyBool, PyFloat, PyInt, PyNone};
use serde_json::{Value, Map};
use std::collections::HashMap;

/// Convert a Python object to a JSON Value
pub fn py_object_to_json_value(py: Python<'_>, obj: PyObject) -> PyResult<Value> {
    if obj.is_none(py) {
        return Ok(Value::Null);
    }

    if let Ok(val) = obj.extract::<bool>(py) {
        return Ok(Value::Bool(val));
    }

    if let Ok(val) = obj.extract::<i64>(py) {
        return Ok(Value::Number(val.into()));
    }

    if let Ok(val) = obj.extract::<f64>(py) {
        return Ok(serde_json::Number::from_f64(val)
            .map(Value::Number)
            .unwrap_or(Value::Null));
    }

    if let Ok(val) = obj.extract::<String>(py) {
        return Ok(Value::String(val));
    }

    if let Ok(dict) = obj.extract::<&PyDict>(py) {
        let mut map = Map::new();
        for (key, value) in dict.iter() {
            let key_str = key.extract::<String>()?;
            let value_json = py_object_to_json_value(py, value.to_object(py))?;
            map.insert(key_str, value_json);
        }
        return Ok(Value::Object(map));
    }

    if let Ok(list) = obj.extract::<&PyList>(py) {
        let mut vec = Vec::new();
        for item in list.iter() {
            let value_json = py_object_to_json_value(py, item.to_object(py))?;
            vec.push(value_json);
        }
        return Ok(Value::Array(vec));
    }

    // If we couldn't convert it to a JSON type, use string representation
    let str_repr = obj.extract::<String>(py)?;
    Ok(Value::String(str_repr))
}

/// Convert a JSON Value to a Python object
pub fn json_value_to_py_object(py: Python<'_>, value: Value) -> PyResult<PyObject> {
    match value {
        Value::Null => Ok(py.None()),
        Value::Bool(b) => Ok(PyBool::new(py, b).into()),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i.to_object(py))
            } else if let Some(f) = n.as_f64() {
                Ok(PyFloat::new(py, f).into())
            } else {
                Ok(PyString::new(py, &n.to_string()).into())
            }
        }
        Value::String(s) => Ok(PyString::new(py, &s).into()),
        Value::Array(arr) => {
            let py_list = PyList::empty(py);
            for item in arr {
                let py_item = json_value_to_py_object(py, item)?;
                py_list.append(py_item)?;
            }
            Ok(py_list.into())
        }
        Value::Object(obj) => {
            let py_dict = PyDict::new(py);
            for (key, val) in obj {
                let py_val = json_value_to_py_object(py, val)?;
                py_dict.set_item(key, py_val)?;
            }
            Ok(py_dict.into())
        }
    }
}

/// Convert a HashMap to a Python dictionary
pub fn hashmap_to_py_dict<K, V>(py: Python<'_>, map: &HashMap<K, V>) -> PyResult<Py<PyDict>>
where
    K: ToPyObject + std::fmt::Display,
    V: ToPyObject,
{
    let dict = PyDict::new(py);
    for (key, value) in map {
        dict.set_item(key.to_string(), value.to_object(py))?;
    }
    Ok(dict.into())
}

/// Convert a Python dictionary to a HashMap
pub fn py_dict_to_hashmap<'py, V>(py: Python<'py>, dict: &'py PyDict) -> PyResult<HashMap<String, V>>
where
    V: FromPyObject<'py>,
{
    let mut map = HashMap::new();
    for (key, value) in dict.iter() {
        let key_str = key.extract::<String>()?;
        let val = value.extract::<V>()?;
        map.insert(key_str, val);
    }
    Ok(map)
}

/// Convert to a string-only HashMap
pub fn py_dict_to_string_map<'py>(py: Python<'py>, dict: &'py PyDict) -> PyResult<HashMap<String, String>> {
    let mut map = HashMap::new();
    for (key, value) in dict.iter() {
        let key_str = key.extract::<String>()?;
        let val_str = value.extract::<String>()?;
        map.insert(key_str, val_str);
    }
    Ok(map)
} 
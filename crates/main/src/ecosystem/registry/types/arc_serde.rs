// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Serde helpers for [`Arc<str>`](std::sync::Arc) and related maps used by registry DTOs.

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

pub fn serialize_arc_str<S>(arc_str: &Arc<str>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(arc_str)
}

pub fn deserialize_arc_str<'de, D>(deserializer: D) -> Result<Arc<str>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(Arc::from(s))
}

pub fn serialize_arc_str_vec<S>(vec: &[Arc<str>], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let strings: Vec<&str> = vec.iter().map(std::convert::AsRef::as_ref).collect();
    strings.serialize(serializer)
}

pub fn deserialize_arc_str_vec<'de, D>(deserializer: D) -> Result<Vec<Arc<str>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let strings = Vec::<String>::deserialize(deserializer)?;
    Ok(strings.into_iter().map(Arc::from).collect())
}

pub fn serialize_arc_str_map<S>(
    map: &HashMap<Arc<str>, Arc<str>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let string_map: HashMap<&str, &str> =
        map.iter().map(|(k, v)| (k.as_ref(), v.as_ref())).collect();
    string_map.serialize(serializer)
}

pub fn deserialize_arc_str_map<'de, D>(
    deserializer: D,
) -> Result<HashMap<Arc<str>, Arc<str>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let string_map = HashMap::<String, String>::deserialize(deserializer)?;
    Ok(string_map
        .into_iter()
        .map(|(k, v)| (Arc::from(k), Arc::from(v)))
        .collect())
}

// Serde passes &Option<T> for serialize_with
#[expect(clippy::ref_option, reason = "Optional reference; API design")]
pub fn serialize_optional_arc_str<S>(
    opt: &Option<Arc<str>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match opt {
        Some(arc_str) => serializer.serialize_some(arc_str.as_ref()),
        None => serializer.serialize_none(),
    }
}

pub fn deserialize_optional_arc_str<'de, D>(deserializer: D) -> Result<Option<Arc<str>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt_string = Option::<String>::deserialize(deserializer)?;
    Ok(opt_string.map(Arc::from))
}

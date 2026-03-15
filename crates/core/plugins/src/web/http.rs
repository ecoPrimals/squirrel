// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! HTTP utilities for web plugins
//!
//! This module provides HTTP-related utilities for web plugins.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// HTTP methods supported by the plugin system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HttpMethod {
    /// GET method
    Get,
    /// POST method
    Post,
    /// PUT method
    Put,
    /// DELETE method
    Delete,
    /// PATCH method
    Patch,
    /// OPTIONS method
    Options,
    /// HEAD method
    Head,
}

impl HttpMethod {
    /// Check if the method is GET
    pub fn is_get(&self) -> bool {
        *self == HttpMethod::Get
    }

    /// Check if the method is POST
    pub fn is_post(&self) -> bool {
        *self == HttpMethod::Post
    }

    /// Check if the method is PUT
    pub fn is_put(&self) -> bool {
        *self == HttpMethod::Put
    }

    /// Check if the method is DELETE
    pub fn is_delete(&self) -> bool {
        *self == HttpMethod::Delete
    }

    /// Check if the method is PATCH
    pub fn is_patch(&self) -> bool {
        *self == HttpMethod::Patch
    }

    /// Check if the method is OPTIONS
    pub fn is_options(&self) -> bool {
        *self == HttpMethod::Options
    }

    /// Check if the method is HEAD
    pub fn is_head(&self) -> bool {
        *self == HttpMethod::Head
    }

    /// Check if the method is safe (doesn't modify resources)
    pub fn is_safe(&self) -> bool {
        matches!(
            self,
            HttpMethod::Get | HttpMethod::Head | HttpMethod::Options
        )
    }

    /// Check if the method is idempotent (can be called multiple times with same effect)
    pub fn is_idempotent(&self) -> bool {
        matches!(
            self,
            HttpMethod::Get
                | HttpMethod::Head
                | HttpMethod::Options
                | HttpMethod::Put
                | HttpMethod::Delete
        )
    }
}

impl fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpMethod::Get => write!(f, "GET"),
            HttpMethod::Post => write!(f, "POST"),
            HttpMethod::Put => write!(f, "PUT"),
            HttpMethod::Delete => write!(f, "DELETE"),
            HttpMethod::Patch => write!(f, "PATCH"),
            HttpMethod::Options => write!(f, "OPTIONS"),
            HttpMethod::Head => write!(f, "HEAD"),
        }
    }
}

impl FromStr for HttpMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(HttpMethod::Get),
            "POST" => Ok(HttpMethod::Post),
            "PUT" => Ok(HttpMethod::Put),
            "DELETE" => Ok(HttpMethod::Delete),
            "PATCH" => Ok(HttpMethod::Patch),
            "OPTIONS" => Ok(HttpMethod::Options),
            "HEAD" => Ok(HttpMethod::Head),
            _ => Err(format!("Unknown HTTP method: {}", s)),
        }
    }
}

/// HTTP status codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HttpStatus {
    /// 200 OK
    Ok = 200,
    /// 201 Created
    Created = 201,
    /// 202 Accepted
    Accepted = 202,
    /// 204 No Content
    NoContent = 204,
    /// 400 Bad Request
    BadRequest = 400,
    /// 401 Unauthorized
    Unauthorized = 401,
    /// 403 Forbidden
    Forbidden = 403,
    /// 404 Not Found
    NotFound = 404,
    /// 405 Method Not Allowed
    MethodNotAllowed = 405,
    /// 409 Conflict
    Conflict = 409,
    /// 500 Internal Server Error
    InternalServerError = 500,
    /// 501 Not Implemented
    NotImplemented = 501,
    /// 503 Service Unavailable
    ServiceUnavailable = 503,
}

impl HttpStatus {
    /// Get the status code
    pub fn code(&self) -> u16 {
        *self as u16
    }

    /// Check if the status is successful (2xx)
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.code())
    }

    /// Check if the status is an error (4xx or 5xx)
    pub fn is_error(&self) -> bool {
        self.code() >= 400
    }

    /// Check if the status is a client error (4xx)
    pub fn is_client_error(&self) -> bool {
        (400..500).contains(&self.code())
    }

    /// Check if the status is a server error (5xx)
    pub fn is_server_error(&self) -> bool {
        self.code() >= 500
    }
}

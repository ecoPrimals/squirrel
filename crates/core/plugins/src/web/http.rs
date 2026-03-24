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
    #[must_use]
    pub fn is_get(&self) -> bool {
        *self == Self::Get
    }

    /// Check if the method is POST
    #[must_use]
    pub fn is_post(&self) -> bool {
        *self == Self::Post
    }

    /// Check if the method is PUT
    #[must_use]
    pub fn is_put(&self) -> bool {
        *self == Self::Put
    }

    /// Check if the method is DELETE
    #[must_use]
    pub fn is_delete(&self) -> bool {
        *self == Self::Delete
    }

    /// Check if the method is PATCH
    #[must_use]
    pub fn is_patch(&self) -> bool {
        *self == Self::Patch
    }

    /// Check if the method is OPTIONS
    #[must_use]
    pub fn is_options(&self) -> bool {
        *self == Self::Options
    }

    /// Check if the method is HEAD
    #[must_use]
    pub fn is_head(&self) -> bool {
        *self == Self::Head
    }

    /// Check if the method is safe (doesn't modify resources)
    #[must_use]
    pub const fn is_safe(&self) -> bool {
        matches!(self, Self::Get | Self::Head | Self::Options)
    }

    /// Check if the method is idempotent (can be called multiple times with same effect)
    #[must_use]
    pub const fn is_idempotent(&self) -> bool {
        matches!(
            self,
            Self::Get | Self::Head | Self::Options | Self::Put | Self::Delete
        )
    }
}

impl fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Get => write!(f, "GET"),
            Self::Post => write!(f, "POST"),
            Self::Put => write!(f, "PUT"),
            Self::Delete => write!(f, "DELETE"),
            Self::Patch => write!(f, "PATCH"),
            Self::Options => write!(f, "OPTIONS"),
            Self::Head => write!(f, "HEAD"),
        }
    }
}

impl FromStr for HttpMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(Self::Get),
            "POST" => Ok(Self::Post),
            "PUT" => Ok(Self::Put),
            "DELETE" => Ok(Self::Delete),
            "PATCH" => Ok(Self::Patch),
            "OPTIONS" => Ok(Self::Options),
            "HEAD" => Ok(Self::Head),
            _ => Err(format!("Unknown HTTP method: {s}")),
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
    #[must_use]
    pub const fn code(&self) -> u16 {
        *self as u16
    }

    /// Check if the status is successful (2xx)
    #[must_use]
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.code())
    }

    /// Check if the status is an error (4xx or 5xx)
    #[must_use]
    pub const fn is_error(&self) -> bool {
        self.code() >= 400
    }

    /// Check if the status is a client error (4xx)
    #[must_use]
    pub fn is_client_error(&self) -> bool {
        (400..500).contains(&self.code())
    }

    /// Check if the status is a server error (5xx)
    #[must_use]
    pub const fn is_server_error(&self) -> bool {
        self.code() >= 500
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_method_display() {
        assert_eq!(HttpMethod::Get.to_string(), "GET");
        assert_eq!(HttpMethod::Post.to_string(), "POST");
        assert_eq!(HttpMethod::Put.to_string(), "PUT");
        assert_eq!(HttpMethod::Delete.to_string(), "DELETE");
        assert_eq!(HttpMethod::Patch.to_string(), "PATCH");
        assert_eq!(HttpMethod::Options.to_string(), "OPTIONS");
        assert_eq!(HttpMethod::Head.to_string(), "HEAD");
    }

    #[test]
    fn http_method_from_str_case_insensitive() {
        assert_eq!(
            "get".parse::<HttpMethod>().expect("should succeed"),
            HttpMethod::Get
        );
        assert_eq!(
            "POST".parse::<HttpMethod>().expect("should succeed"),
            HttpMethod::Post
        );
        assert!(HttpMethod::from_str("UNKNOWN").is_err());
    }

    #[test]
    fn http_method_predicates() {
        assert!(HttpMethod::Get.is_get());
        assert!(HttpMethod::Post.is_post());
        assert!(HttpMethod::Put.is_put());
        assert!(HttpMethod::Delete.is_delete());
        assert!(HttpMethod::Patch.is_patch());
        assert!(HttpMethod::Options.is_options());
        assert!(HttpMethod::Head.is_head());
        assert!(HttpMethod::Get.is_safe());
        assert!(HttpMethod::Head.is_safe());
        assert!(HttpMethod::Options.is_safe());
        assert!(!HttpMethod::Post.is_safe());
        assert!(HttpMethod::Put.is_idempotent());
        assert!(HttpMethod::Delete.is_idempotent());
        assert!(!HttpMethod::Post.is_idempotent());
    }

    #[test]
    fn http_method_serde_roundtrip() {
        let methods = [
            HttpMethod::Get,
            HttpMethod::Post,
            HttpMethod::Put,
            HttpMethod::Delete,
            HttpMethod::Patch,
            HttpMethod::Options,
            HttpMethod::Head,
        ];
        for m in methods {
            let j = serde_json::to_string(&m).expect("should succeed");
            let back: HttpMethod = serde_json::from_str(&j).expect("should succeed");
            assert_eq!(back, m);
        }
    }

    #[test]
    fn http_status_helpers_and_serde() {
        assert!(HttpStatus::Ok.is_success());
        assert!(!HttpStatus::Ok.is_error());
        assert!(!HttpStatus::Ok.is_client_error());
        assert!(!HttpStatus::Ok.is_server_error());

        assert!(HttpStatus::NotFound.is_error());
        assert!(HttpStatus::NotFound.is_client_error());
        assert!(!HttpStatus::NotFound.is_success());

        assert!(HttpStatus::InternalServerError.is_error());
        assert!(HttpStatus::InternalServerError.is_server_error());
        assert!(!HttpStatus::InternalServerError.is_client_error());

        let s = HttpStatus::Created;
        let j = serde_json::to_string(&s).expect("should succeed");
        let back: HttpStatus = serde_json::from_str(&j).expect("should succeed");
        assert_eq!(back, s);
        assert_eq!(back.code(), 201);
    }
}

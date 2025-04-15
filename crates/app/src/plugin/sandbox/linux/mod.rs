#![cfg(target_family = "unix")]

//! Linux-specific plugin sandbox implementation using cgroups v2
//!
//! This module provides a Linux-specific implementation of the PluginSandbox trait
//! using cgroups v2 for process isolation and resource limits.

mod config;
mod sandbox;
mod utils;
mod trait_impl;
mod seccomp;
mod resources;
mod sandbox_io;

pub use config::SeccompConfig;
pub use sandbox::LinuxCgroupSandbox;
pub use utils::*;

#[cfg(test)]
mod tests; 
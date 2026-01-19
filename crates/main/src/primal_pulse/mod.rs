//! PrimalPulse - AI-Powered Ecosystem Intelligence
//!
//! **LEGACY MODULE** - being evolved to capability-based architecture
//!
//! TODO: Rebuild using capability_ai instead of deleted HTTP API

// Legacy modules REMOVED - used deleted HTTP API (api::ai)
// pub(crate) mod handlers; // DELETED
// mod tools;                // DELETED  
// pub use tools::register_primal_pulse_tools; // DELETED

// Remaining modules (may need updates)
pub mod neural_graph;
mod schemas;

#[cfg(test)]
mod tests;

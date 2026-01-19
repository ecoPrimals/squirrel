// AI routing and provider selection modules

pub(crate) mod action_registry;
pub(crate) mod adapters;
pub(crate) mod constraint_router;
pub(crate) mod constraints;
pub(crate) mod router;
pub(crate) mod selector;
pub(crate) mod types;

// Re-export main router for tarpc_server
pub use router::AiRouter;


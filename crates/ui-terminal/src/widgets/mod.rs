// Export widgets from this module
pub mod alerts;
pub mod chart;
pub mod connection_health;
pub mod health;
pub mod metrics;
pub mod network;
pub mod protocol;
pub mod system;

// Optional: Define a common Widget trait if needed
// use ratatui::{
//     backend::Backend,
//     layout::Rect,
//     Frame,
// };
// use crate::app::App;
//
// pub trait Widget {
//     fn render<B: Backend>(frame: &mut Frame<'_, B>, app: &App, area: Rect);
// } 
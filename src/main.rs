//! Snippet sharing server.

#![warn(missing_docs)]

mod app_error;
mod app_state;
mod auth_user;
mod database;
mod handlers;
mod middleware;
mod models;
mod server;
mod web;

pub(crate) use app_state::AppState;
pub use server::main;

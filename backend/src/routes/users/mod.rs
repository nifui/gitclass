pub mod controllers;
pub mod models;
pub mod repository;
pub mod services;

pub use controllers::router;
use serde::{Deserialize, Serialize};
use thiserror::Error;
// Routes for basic user info, like their classes and general grade for each class.
#[derive(Error, Serialize, Deserialize, Debug)]
pub enum UserError {
    #[error("Placeholder")]
    Placeholder,
}

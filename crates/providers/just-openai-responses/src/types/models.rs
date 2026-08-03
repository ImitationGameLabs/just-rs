//! Model-listing DTOs for `GET /models`.
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ListModelsResponse {
    pub object: String,
    pub data: Vec<Model>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Model {
    pub id: String,
    pub object: String,
    pub owned_by: String,
}

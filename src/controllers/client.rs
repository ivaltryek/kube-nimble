use std::sync::Arc;

use kube::{runtime::controller::Action, Client};
use tokio::time::Duration;

use thiserror::Error;

use crate::crds::nimble::Nimble;

pub struct ContextData {
    pub client: Client,
}

impl ContextData {
    pub fn new(client: Client) -> Self {
        ContextData { client }
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to create Nimble Object: {0}")]
    NimbleObjectCreationFailed(#[source] kube::Error),
    #[error("MissingObjectKey: {0}")]
    MissingObjectKey(&'static str),
}

pub fn error_policy(_object: Arc<Nimble>, _error: &Error, _ctx: Arc<ContextData>) -> Action {
    Action::requeue(Duration::from_secs(1))
}

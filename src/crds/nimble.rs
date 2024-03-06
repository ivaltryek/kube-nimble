use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{deploymentspec::DeploySpec, servicespec::SvcSpec};

#[derive(kube::CustomResource, Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
#[kube(
    group = "ivaltryek.github.com",
    version = "v1",
    kind = "Nimble",
    plural = "nimbles",
    derive = "PartialEq",
    namespaced
)]

pub struct NimbleSpec {
    pub deployment: DeploySpec,
    pub service: Option<SvcSpec>,
}

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{deploymentspec::DeploySpec, hpaspec::HPASpec, ingspec::IngSpec, servicespec::SvcSpec};

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
    #[doc = "Spec for Deployment Object"]
    pub deployment: DeploySpec,
    #[doc = "Spec for Service Object"]
    pub service: Option<SvcSpec>,
    #[doc = "Spec for Autoscaling (HPA) Object"]
    pub hpa: Option<HPASpec>,
    #[doc = "Spec for Ingress Object"]
    pub ingress: Option<IngSpec>,
}

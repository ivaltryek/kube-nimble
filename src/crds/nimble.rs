use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub struct DeploySpec {
    #[doc = "Container image to use for the deployment."]
    pub image: String,
    #[doc = "Number of desired replicas for the deployment."]
    #[serde(default = "default_replicas")]
    pub replicas: i32,
    #[doc = "Labels to be applied to the deployment and its pods."]
    pub labels: BTreeMap<String, String>,
    #[doc = "Annotations to be applied to the deployment and its pods."]
    #[serde(default = "default_annotations")]
    pub annotations: Option<BTreeMap<String, String>>,
}

/**
 * This function returns the default value for the number of replicas.
 * In this specific case, the default is set to 1.
 *
 * You can customize this value by modifying the function body.
 */
pub fn default_replicas() -> i32 {
    1
}

/**
 * This function creates a default `Option<BTreeMap<String, String>>` containing a single key-value pair:
 *   - "app.kubernetes.io/managed-by": "kube-nimble"
 *
 * This annotation can be used to identify resources managed by the kube-nimble tool.
 *
 * The function returns `None` if there is an issue creating the `BTreeMap`.
 */
pub fn default_annotations() -> Option<BTreeMap<String, String>> {
    let mut annotations = Some(BTreeMap::new());
    match &mut annotations {
        Some(map) => {
            map.insert(
                "app.kubernetes.io/managed-by".to_owned(),
                "kube-nimble".to_owned(),
            );
        }
        None => {
            tracing::warn!(msg = "Map is not yet initialized.");
        }
    }
    annotations
}

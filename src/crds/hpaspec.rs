use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::deploymentspec::default_annotations;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub struct HPASpec {
    #[doc = "Annotations to be applied to the HPA object"]
    #[serde(default = "default_hpa_annotations")]
    pub annotations: Option<BTreeMap<String, String>>,
    #[doc = "maxReplicas is the upper limit for the number of replicas to which the autoscaler can scale up.
      It cannot be less that minReplicas."]
    pub max: i32,
    #[doc = "minReplicas is the lower limit for the number of replicas to which the autoscaler can scale down.
     It defaults to 1 pod.
     minReplicas is allowed to be 0 if the alpha feature gate HPAScaleToZero is enabled and at least one Object or External metric is configured.
     Scaling is active as long as at least one metric value is available."]
    pub min: Option<i32>,
    #[doc = "resource refers to a resource metric (such as those specified in requests and limits) 
      known to Kubernetes describing each pod in the current scale target (e.g. CPU or memory)."]
    #[serde(rename = "resourcePolicy")]
    pub resource_policy: Option<ResourceMetricSpec>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub struct ResourceMetricSpec {
    #[doc = "name is the name of the resource in question."]
    pub name: String,
    #[doc = "type represents whether the metric type is Utilization, Value, or AverageValue"]
    #[serde(rename = "type")]
    pub type_: String,
    #[doc = "avgUtil is the target value of the average of the resource metric across all relevant pods, 
      represented as a percentage of the requested value of the resource for the pods.
      Currently only valid for Resource metric source type"]
    #[serde(rename = "avgUtil")]
    pub average_utilization: Option<i32>,
}

// Return default annotations to applied to an object.
fn default_hpa_annotations() -> Option<BTreeMap<String, String>> {
    default_annotations()
}

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub struct DeploySpec {
    #[doc = "Containers to run in the deployment."]
    pub containers: Vec<ContainerSpec>,
    #[doc = "Number of desired replicas for the deployment."]
    #[serde(default = "default_replicas")]
    pub replicas: i32,
    #[doc = "Labels to be applied to the deployment and its pods."]
    pub labels: BTreeMap<String, String>,
    #[doc = "Annotations to be applied to the deployment and its pods."]
    #[serde(default = "default_annotations")]
    pub annotations: Option<BTreeMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub struct ContainerSpec {
    #[doc = "Name of the container."]
    pub name: String,
    #[doc = "Image to use for a container."]
    pub image: String,
    #[doc = "override entrypoint command for a container."]
    pub command: Option<Vec<String>>,
    #[doc = "cpu request for a container."]
    #[serde(rename = "cpuRequest", default = "default_cpu_resource_requests")]
    pub cpu_request: String,
    #[doc = "cpu limit for a container."]
    #[serde(rename = "cpuLimit", default = "default_cpu_resource_limits")]
    pub cpu_limit: String,
    #[doc = "memory request for a container."]
    #[serde(rename = "memoryRequest", default = "default_memory_resource_requests")]
    pub memory_request: String,
    #[doc = "memory limit for a container."]
    #[serde(rename = "memoryLimit", default = "default_memory_resource_limits")]
    pub memory_limit: String,
    #[doc = "Periodic probe of container service readiness. 
      Container will be removed from service endpoints if the probe fails. 
      Cannot be updated. More info: https://kubernetes.io/docs/concepts/workloads/pods/pod-lifecycle#container-probes"]
    #[serde(rename = "readinessProbe")]
    pub readiness_probe: Option<ProbeSpec>,
    #[doc = "Periodic probe of container liveness. Container will be restarted if the probe fails. 
      Cannot be updated. More info: https://kubernetes.io/docs/concepts/workloads/pods/pod-lifecycle#container-probes"]
    #[serde(rename = "livenessProbe")]
    pub liveness_probe: Option<ProbeSpec>,
    #[doc = "StartupProbe indicates that the Pod has successfully initialized. 
      If specified, no other probes are executed until this completes successfully. 
      If this probe fails, the Pod will be restarted, just as if the livenessProbe failed. 
      This can be used to provide different probe parameters at the beginning of a Pod’s lifecycle, 
      when it might take a long time to load data or warm a cache, than during steady-state operation. 
      This cannot be updated. More info: https://kubernetes.io/docs/concepts/workloads/pods/pod-lifecycle#container-probes"]
    #[serde(rename = "startupProbe")]
    pub startup_probe: Option<ProbeSpec>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub struct ProbeSpec {
    #[doc = "Exec specifies the action to take."]
    pub exec: Option<Vec<String>>,
    #[doc = "HTTPGet specifies the http request to perform."]
    #[serde(rename = "httpGet")]
    pub http_get: Option<HTTPGet>,
    #[doc = "Number of seconds after the container has started before liveness probes are initiated. 
      More info: https://kubernetes.io/docs/concepts/workloads/pods/pod-lifecycle#container-probes"]
    #[serde(
        rename = "initialDelaySeconds",
        default = "default_initial_delay_seconds"
    )]
    pub initial_delay_seconds: Option<i32>,
    #[doc = "How often (in seconds) to perform the probe. Default to 10 seconds. Minimum value is 1."]
    #[serde(rename = "periodSeconds", default = "default_period_seconds")]
    pub period_seconds: Option<i32>,
    #[doc = "Minimum consecutive successes for the probe to be considered successful after having failed. 
      Defaults to 1. Must be 1 for liveness and startup. Minimum value is 1."]
    #[serde(rename = "successThreshold", default = "default_success_threshold")]
    pub success_threshold: Option<i32>,
    #[doc = "TCPSocket specifies an action involving a TCP port."]
    #[serde(rename = "tcpSocket")]
    pub tcp_socket: Option<TCPSocket>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub struct HTTPGet {
    #[doc = "Path to access on the HTTP server."]
    pub path: String,
    #[doc = "Name or number of the port to access on the container. Number must be in the range 1 to 65535."]
    pub port: i32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub struct TCPSocket {
    #[doc = "TCP Port to make checks against."]
    pub port: i32,
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

/* This function creates a default `Option<BTreeMap<String, String>>` containing a single key-value pair:
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

//This function returns the default value for the cpu_limit field in ContainerSpec.
pub fn default_cpu_resource_limits() -> String {
    String::from("100m")
}

// This function returns the default value for the memory_limit field in ContainerSpec.,
pub fn default_memory_resource_limits() -> String {
    String::from("100Mi")
}

// This function returns the default value for cpu_request field in ContainerSpec.
pub fn default_cpu_resource_requests() -> String {
    String::from("50m")
}

// This function returns the default value for memory_request field in ContainerSpec.
pub fn default_memory_resource_requests() -> String {
    String::from("50Mi")
}

// This function returns the default value for initial_delay_seconds field in ProbeSpec.
pub fn default_initial_delay_seconds() -> Option<i32> {
    Some(0)
}

// This function returns the default value for default_period_seconds field in ProbeSpec.
pub fn default_period_seconds() -> Option<i32> {
    Some(10)
}

// This function returns the default value for default_success_threshold field in ProbeSpec.
pub fn default_success_threshold() -> Option<i32> {
    Some(1)
}

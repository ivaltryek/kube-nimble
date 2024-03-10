use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub struct DeploySpec {
    #[doc = "Containers to run in the deployment."]
    pub containers: Vec<ContainerSpec>,
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
    #[doc = "Requests describes the minimum amount of compute resources required. 
      If Requests is omitted for a container, it defaults to Limits if that is explicitly specified, 
      otherwise to an implementation-defined value. Requests cannot exceed Limits. 
      More info: https://kubernetes.io/docs/concepts/configuration/manage-resources-containers/
    "]
    pub requests: Option<ResourceSpec>,
    #[doc = "Limits describes the maximum amount of compute resources allowed. 
      More info: https://kubernetes.io/docs/concepts/configuration/manage-resources-containers/"]
    pub limits: Option<ResourceSpec>,
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
    #[doc = "List of environment variables to set in the container. Cannot be updated."]
    pub env: Option<Vec<EnvSpec>>,
    #[doc = "List of sources to populate environment variables in the container. 
      The keys defined within a source must be a C_IDENTIFIER.
      All invalid keys will be reported as an event when the container is starting.
      When a key exists in multiple sources, the value associated with the last source will take precedence.
      Values defined by an Env with a duplicate key will take precedence. Cannot be updated."]
    #[serde(rename = "envFrom")]
    pub env_from: Option<Vec<EnvFromSpec>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub struct ResourceSpec {
    #[doc = "cpu config (requests/limits) for the container."]
    pub cpu: Option<String>,
    #[doc = "memory config (requests/limits) for the container."]
    pub memory: Option<String>,
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub struct EnvSpec {
    #[doc = "Name of environment variable"]
    pub name: String,
    #[doc = "Variable references $(VAR_NAME) are expanded using the previously defined environment variables in the container and any service environment variables. 
      If a variable cannot be resolved, the reference in the input string will be unchanged. 
      Double $$ are reduced to a single $, which allows for escaping the $(VAR_NAME)
      syntax: i.e. “$$(VAR_NAME)” will produce the string literal “$(VAR_NAME)”. 
      Escaped references will never be expanded, regardless of whether the variable exists or not. Defaults to “”."]
    pub value: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub struct EnvFromSpec {
    #[doc = "The ConfigMap to select from"]
    #[serde(rename = "configMapRef")]
    pub config_map_ref: Option<String>,
    #[doc = "The Secret to select from"]
    #[serde(rename = "secretRef")]
    pub secret_ref: Option<String>,
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

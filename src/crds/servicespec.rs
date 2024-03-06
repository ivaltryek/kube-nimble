use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::deploymentspec::default_annotations;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub struct SvcSpec {
    #[doc = "Annotations to be applied to the service."]
    #[serde(default = "default_svc_annotations")]
    pub annotations: Option<BTreeMap<String, String>>,
    #[doc = "Route service traffic to pods with label keys and values matching this selector.
     If empty or not present, the service is assumed to have an external process managing its endpoints,
     which Kubernetes will not modify. 
     Only applies to types ClusterIP, NodePort, and LoadBalancer.
     Ignored if type is ExternalName. More info: https://kubernetes.io/docs/concepts/services-networking/service/"]
    pub selector: Option<BTreeMap<String, String>>,
    #[doc = "type determines how the Service is exposed. 
      Defaults to ClusterIP. 
      Valid options are ExternalName, ClusterIP, NodePort, and LoadBalancer.
      “ClusterIP” allocates a cluster-internal IP address for load-balancing to endpoints.
      Endpoints are determined by the selector or if that is not specified, by manual construction of an Endpoints object or EndpointSlice objects.
      If clusterIP is “None”, no virtual IP is allocated and the endpoints are published as a set of endpoints rather than a virtual IP.
      “NodePort” builds on ClusterIP and allocates a port on every node which routes to the same endpoints as the clusterIP.
      “LoadBalancer” builds on NodePort and creates an external load-balancer (if supported in the current cloud) 
      which routes to the same endpoints as the clusterIP. “ExternalName” aliases this service to the specified externalName.
      Several other fields do not apply to ExternalName services. 
      More info: https://kubernetes.io/docs/concepts/services-networking/service/#publishing-services-service-types"]
    #[serde(rename = "type")]
    pub type_: Option<String>,
    #[doc = "The list of ports that are exposed by this service. 
      More info: https://kubernetes.io/docs/concepts/services-networking/service/#virtual-ips-and-service-proxies"]
    pub ports: Option<Vec<PortSpec>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub struct PortSpec {
    #[doc = "The name of this port within the service.
      This must be a DNS_LABEL. All ports within a ServiceSpec must have unique names.
      When considering the endpoints for a Service, this must match the ‘name’ field in the EndpointPort.
      Optional if only one ServicePort is defined on this service."]
    pub name: Option<String>,
    #[doc = "The port on each node on which this service is exposed when type is NodePort or LoadBalancer.
      Usually assigned by the system. If a value is specified, in-range, and not in use it will be used,
      otherwise the operation will fail. If not specified, a port will be allocated if this Service requires one.
      If this field is specified when creating a Service which does not need it, creation will fail.
      This field will be wiped when updating a Service to no longer need it (e.g. changing type from NodePort to ClusterIP).
      More info: https://kubernetes.io/docs/concepts/services-networking/service/#type-nodeport"]
    #[serde(rename = "nodePort")]
    pub node_port: Option<i32>,
    #[doc = "The port that will be exposed by this service."]
    pub port: i32,
    #[doc = "The IP protocol for this port. Supports “TCP”, “UDP”, and “SCTP”. Default is TCP."]
    #[serde(default = "default_protocol")]
    pub protocol: Option<String>,
    #[doc = "Number or name of the port to access on the pods targeted by the service.
      Number must be in the range 1 to 65535. Name must be an IANA_SVC_NAME.
      If this is a string, it will be looked up as a named port in the target Pod’s container ports. 
      If this is not specified, the value of the ‘port’ field is used (an identity map). 
      This field is ignored for services with clusterIP=None, and should be omitted or set equal to the ‘port’ field. 
      More info: https://kubernetes.io/docs/concepts/services-networking/service/#defining-a-service"]
    #[serde(rename = "target_port")]
    pub target_port: Option<i32>,
}

fn default_svc_annotations() -> Option<BTreeMap<String, String>> {
    default_annotations()
}

fn default_protocol() -> Option<String> {
    Some("TCP".to_owned())
}

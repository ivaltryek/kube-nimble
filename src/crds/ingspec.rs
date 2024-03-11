use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::deploymentspec::default_annotations;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub struct IngSpec {
    #[doc = "annotations to be applied on Ingress Object"]
    #[serde(default = "default_ing_annotations")]
    pub annotations: Option<BTreeMap<String, String>>,
    #[doc = "ingressClassName is the name of an IngressClass cluster resource. 
     Ingress controller implementations use this field to know whether they should be serving this Ingress resource, 
     by a transitive connection (controller -> IngressClass -> Ingress resource).
     Although the kubernetes.io/ingress.class annotation (simple constant name) was never formally defined,
     it was widely supported by Ingress controllers to create a direct binding between Ingress controller and Ingress resources.
     Newly created Ingress resources should prefer using the field. However, 
     even though the annotation is officially deprecated, for backwards compatibility reasons, 
     ingress controllers should still honor that annotation if present."]
    pub class: Option<String>,
    #[doc = "rules is a list of host rules used to configure the Ingress. If unspecified, or no rule matches, 
     all traffic is sent to the default backend."]
    pub rules: Option<Vec<RuleSpec>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub struct RuleSpec {
    #[doc = "host can be “precise” which is a domain name without the terminating dot of a network host (e.g. “foo.bar.com”) or “wildcard”,
     which is a domain name prefixed with a single wildcard label (e.g. “.foo.com”). 
     The wildcard character ‘’ must appear by itself as the first DNS label and matches only a single label. 
     You cannot have a wildcard label by itself (e.g. Host == “*”). Requests will be matched against the Host field in the following way: 1. 
     If host is precise, the request matches this rule if the http host header is equal to Host. 2. 
     If host is a wildcard, then the request matches this rule if the http host header is to equal to the suffix (removing the first label) of the wildcard rule."]
    pub host: Option<String>,
    #[doc = "pathType determines the interpretation of the path matching. 
      PathType can be one of the following values: * Exact: Matches the URL path exactly. 
      * Prefix: Matches based on a URL path prefix split by ‘/’. Matching is done on a path element by element basis.
      A path element refers is the list of labels in the path split by the ‘/’ separator.
      A request is a match for path p if every p is an element-wise prefix of p of the request path.
      Note that if the last element of the path is a substring of the last element in request path, 
      it is not a match (e.g. /foo/bar matches /foo/bar/baz, but does not match /foo/barbaz)."]
    #[serde(rename = "pathType")]
    pub path_type: String,
    #[doc = "path is matched against the path of an incoming request.
      Currently it can contain characters disallowed from the conventional “path” part of a URL as defined by RFC 3986. 
      Paths must begin with a ‘/’ and must be present when using PathType with value “Exact” or “Prefix”."]
    pub path: Option<String>,
    #[doc = "port of the referenced service. A port name or port number is required for a IngressServiceBackend."]
    pub port: Option<i32>,
}

fn default_ing_annotations() -> Option<BTreeMap<String, String>> {
    default_annotations()
}

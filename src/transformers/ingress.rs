use k8s_openapi::api::networking::v1::{
    HTTPIngressPath, HTTPIngressRuleValue, IngressBackend, IngressRule, IngressServiceBackend,
    ServiceBackendPort,
};

use crate::crds::ingspec::RuleSpec;

pub fn transform_rules(
    rules_spec: Option<Vec<RuleSpec>>,
    svc_name: String,
) -> Option<Vec<IngressRule>> {
    match rules_spec {
        Some(rules) => {
            let mut ingress_rule_vec = Vec::new();
            for rule in rules {
                ingress_rule_vec.push(IngressRule {
                    host: rule.host,
                    http: Some(HTTPIngressRuleValue {
                        paths: vec![HTTPIngressPath {
                            backend: IngressBackend {
                                service: Some(IngressServiceBackend {
                                    name: svc_name.to_owned(),
                                    port: Some(ServiceBackendPort {
                                        number: rule.port,
                                        ..ServiceBackendPort::default()
                                    }),
                                }),
                                ..IngressBackend::default()
                            },
                            path: rule.path,
                            path_type: rule.path_type,
                        }],
                    }),
                });
            }
            Some(ingress_rule_vec)
        }
        _ => None,
    }
}

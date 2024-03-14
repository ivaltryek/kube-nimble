use std::sync::Arc;

use k8s_openapi::api::networking::v1::{
    HTTPIngressPath, HTTPIngressRuleValue, Ingress, IngressBackend, IngressRule,
    IngressServiceBackend, IngressSpec, ServiceBackendPort,
};
use kube::{api::ObjectMeta, Resource};

use crate::crds::{ingspec::RuleSpec, nimble::Nimble};

fn transform_rules(
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

pub fn transform_ingress(nimble: Arc<Nimble>, is_dry_run: bool) -> Ingress {
    let ing_spec = nimble.spec.ingress.clone().unwrap();
    let ingress: Ingress = Ingress {
        metadata: if is_dry_run {
            ObjectMeta {
                name: nimble.metadata.name.clone(),
                annotations: ing_spec.annotations,
                ..ObjectMeta::default()
            }
        } else {
            let oref = nimble.controller_owner_ref(&()).unwrap();
            ObjectMeta {
                name: nimble.metadata.name.clone(),
                owner_references: Some(vec![oref]),
                annotations: ing_spec.annotations,
                ..ObjectMeta::default()
            }
        },
        spec: Some(IngressSpec {
            ingress_class_name: ing_spec.class,
            rules: transform_rules(ing_spec.rules, nimble.metadata.name.clone().unwrap()),
            ..IngressSpec::default()
        }),
        ..Ingress::default()
    };
    ingress
}

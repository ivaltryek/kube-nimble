use std::sync::Arc;

use k8s_openapi::{
    api::core::v1::{Service, ServicePort, ServiceSpec},
    apimachinery::pkg::util::intstr::IntOrString,
};
use kube::{api::ObjectMeta, Resource};

use crate::crds::{nimble::Nimble, servicespec::PortSpec};

fn transform_ports(ports_vec: Option<Vec<PortSpec>>) -> Option<Vec<ServicePort>> {
    match ports_vec {
        Some(ports) => {
            let mut result_vec = Vec::new();
            for port in ports {
                result_vec.push(ServicePort {
                    name: port.name,
                    node_port: port.node_port,
                    port: port.port,
                    target_port: Some(IntOrString::Int(port.target_port.unwrap())),
                    protocol: port.protocol,
                    ..ServicePort::default()
                })
            }
            Some(result_vec)
        }
        _ => None,
    }
}

pub fn transform_svc(nimble: Arc<Nimble>, is_dry_run: bool) -> Service {
    let svc_spec = nimble.spec.service.clone().unwrap();
    let service: Service = Service {
        metadata: if is_dry_run {
            ObjectMeta {
                name: nimble.metadata.name.clone(),
                annotations: svc_spec.annotations,
                ..ObjectMeta::default()
            }
        } else {
            let oref = nimble.controller_owner_ref(&()).unwrap();
            ObjectMeta {
                name: nimble.metadata.name.clone(),
                owner_references: Some(vec![oref]),
                annotations: svc_spec.annotations,
                ..ObjectMeta::default()
            }
        },
        spec: Some(ServiceSpec {
            type_: svc_spec.type_,
            selector: svc_spec.selector,
            ports: transform_ports(svc_spec.ports),
            ..ServiceSpec::default()
        }),
        ..Service::default()
    };
    service
}

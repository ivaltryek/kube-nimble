use k8s_openapi::{api::core::v1::ServicePort, apimachinery::pkg::util::intstr::IntOrString};

use crate::crds::servicespec::PortSpec;

pub fn transform_ports(ports_vec: Option<Vec<PortSpec>>) -> Option<Vec<ServicePort>> {
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

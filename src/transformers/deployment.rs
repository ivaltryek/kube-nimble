use std::collections::BTreeMap;

use k8s_openapi::{
    api::core::v1::{
        ConfigMapEnvSource, Container, EnvFromSource, EnvVar, ExecAction, HTTPGetAction, Probe,
        ResourceRequirements, SecretEnvSource, TCPSocketAction,
    },
    apimachinery::pkg::{api::resource::Quantity, util::intstr::IntOrString},
};

use crate::crds::deploymentspec::{ContainerSpec, EnvFromSpec, EnvSpec, ProbeSpec, ResourceSpec};

// Transform envFrom field values to acceptable structure.
pub fn transform_env_from(env_from_vec: Option<Vec<EnvFromSpec>>) -> Option<Vec<EnvFromSource>> {
    match env_from_vec {
        Some(env_from) => {
            let mut env_from_vars = Vec::new();

            for var in env_from.iter() {
                match (var.config_map_ref.clone(), var.secret_ref.clone()) {
                    (Some(cm), Some(secret)) => {
                        env_from_vars.push(EnvFromSource {
                            config_map_ref: Some(ConfigMapEnvSource {
                                name: Some(cm.clone()),
                                ..ConfigMapEnvSource::default()
                            }),
                            ..EnvFromSource::default()
                        });
                        env_from_vars.push(EnvFromSource {
                            secret_ref: Some(SecretEnvSource {
                                name: Some(secret),
                                ..SecretEnvSource::default()
                            }),
                            ..EnvFromSource::default()
                        })
                    }
                    (Some(cm), None) => env_from_vars.push(EnvFromSource {
                        config_map_ref: Some(ConfigMapEnvSource {
                            name: Some(cm.clone()),
                            ..ConfigMapEnvSource::default()
                        }),
                        ..EnvFromSource::default()
                    }),
                    (None, Some(secret)) => env_from_vars.push(EnvFromSource {
                        secret_ref: Some(SecretEnvSource {
                            name: Some(secret.clone()),
                            ..SecretEnvSource::default()
                        }),
                        ..EnvFromSource::default()
                    }),
                    _ => {}
                }
            }
            Some(env_from_vars)
        }
        _ => None,
    }
}

// Transform env field values to acceptable structure.
pub fn transform_envs(env_vec: Option<Vec<EnvSpec>>) -> Option<Vec<EnvVar>> {
    match env_vec {
        Some(env) => {
            let mut env_vars = Vec::new();

            for var in env.iter() {
                env_vars.push(EnvVar {
                    name: var.name.clone(),
                    value: var.value.clone(),
                    ..EnvVar::default()
                })
            }
            Some(env_vars)
        }
        _ => None,
    }
}

// Transform resources passed in manifest; This function converts given cpu, memory to
// Option<BTreeMap<String, Quantity>>
pub fn transform_resources(
    resource_spec: &Option<ResourceSpec>,
) -> Option<BTreeMap<String, Quantity>> {
    match resource_spec {
        Some(spec) => match (spec.cpu.clone(), spec.memory.clone()) {
            // case when both cpu and memory are provided.
            (Some(cpu), Some(memory)) => Some(BTreeMap::from([
                ("cpu".to_owned(), Quantity(cpu)),
                ("memory".to_owned(), Quantity(memory)),
            ])),
            // case when only memory is provided.
            (None, Some(memory)) => Some(BTreeMap::from([("memory".to_owned(), Quantity(memory))])),
            // case when only cpu is provided.
            (Some(cpu), None) => Some(BTreeMap::from([("cpu".to_owned(), Quantity(cpu))])),
            // case when none of the field is provided.
            _ => None,
        },
        // It could also happen that, requests or limits is not passed in the manifest.
        _ => None,
    }
}

// Transform probes passed in manifest. i.e liveness, readiness, startup.
pub fn transform_probe(probe_type: &Option<ProbeSpec>) -> Option<Probe> {
    // initialise default &ProbeSpec if probe_type is not None.
    // Meaning, that any of the probes were passed to the configuration.
    match probe_type {
        Some(probe) => {
            let mut shared_probe = Probe {
                period_seconds: probe.period_seconds,
                success_threshold: probe.success_threshold,
                initial_delay_seconds: probe.initial_delay_seconds,
                ..Probe::default()
            };

            match (
                probe.exec.clone(),
                probe.http_get.clone(),
                probe.tcp_socket.clone(),
            ) {
                // checks for the case where exec handler is passed.
                (Some(cmd), None, None) => {
                    shared_probe.exec = Some(ExecAction { command: Some(cmd) });

                    Some(shared_probe)
                }
                // checks for the case where httpGet handler is passed.
                (None, Some(http_get), None) => {
                    shared_probe.http_get = Some(HTTPGetAction {
                        path: Some(http_get.path),
                        port: IntOrString::Int(http_get.port),
                        ..HTTPGetAction::default()
                    });

                    Some(shared_probe)
                }
                // checks fir the case where tcpSocket handler is passed.
                (None, None, Some(tcp_sock)) => {
                    shared_probe.tcp_socket = Some(TCPSocketAction {
                        port: IntOrString::Int(tcp_sock.port),
                        ..TCPSocketAction::default()
                    });

                    Some(shared_probe)
                }
                // Returns none if no handler is passed.
                _ => None,
            }
        }
        // Return none because it might happen that no probes were passed in the configuration.
        _ => None,
    }
}

/// Transforms struct `ContainerSpec` to `Container` Vec that is required in `PodSpec`
/// Returns Vec of `Container`.
/// # Arguments
/// * `container_spec` - A Vec of `ContainerSpec`
pub fn transform_containers(container_spec: Vec<ContainerSpec>) -> Vec<Container> {
    let containers: Vec<Container> = container_spec
        .iter()
        .map(|spec| -> Container {
            let mut container = Container {
                name: spec.name.clone(),
                image: Some(spec.image.clone()),
                command: spec.command.clone(),
                resources: Some(ResourceRequirements {
                    requests: transform_resources(&spec.requests),
                    limits: transform_resources(&spec.limits),
                    ..ResourceRequirements::default()
                }),
                env: transform_envs(spec.env.clone()),
                env_from: transform_env_from(spec.env_from.clone()),
                ..Container::default()
            };

            // configure available probes.
            container.liveness_probe = transform_probe(&spec.liveness_probe);
            container.readiness_probe = transform_probe(&spec.readiness_probe);
            container.startup_probe = transform_probe(&spec.startup_probe);

            // return modified container.
            container
        })
        .collect();

    containers
}

use std::{collections::BTreeMap, sync::Arc};

use k8s_openapi::{
    api::{
        apps::v1::{Deployment, DeploymentSpec},
        core::v1::{
            Container, ExecAction, HTTPGetAction, PodSpec, PodTemplateSpec, Probe,
            ResourceRequirements, TCPSocketAction,
        },
    },
    apimachinery::pkg::{
        api::resource::Quantity, apis::meta::v1::LabelSelector, util::intstr::IntOrString,
    },
};
use kube::{
    api::{ObjectMeta, Patch, PatchParams},
    runtime::{controller::Action, watcher::Config, Controller},
    Api, Resource,
};

use crate::crds::{deploymentspec::ProbeSpec, nimble::Nimble};

use crate::crds::deploymentspec::ContainerSpec;

use crate::common::client::{error_policy, ContextData, Error};

use tokio::time::Duration;

use futures::StreamExt;

use tracing::{error, info};

pub fn transform_probe(probe_type: &Option<ProbeSpec>) -> Option<Probe> {
    if let Some(probe) = probe_type {
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
            (Some(cmd), None, None) => {
                shared_probe.exec = Some(ExecAction { command: Some(cmd) });
                
                Some(shared_probe)
            }
            (None, Some(http_get), None) => {
                shared_probe.http_get = Some(HTTPGetAction {
                    path: Some(http_get.path),
                    port: IntOrString::Int(http_get.port),
                    ..HTTPGetAction::default()
                });

                Some(shared_probe)
            }
            (None, None, Some(tcp_sock)) => {
                shared_probe.tcp_socket = Some(TCPSocketAction {
                    port: IntOrString::Int(tcp_sock.port),
                    ..TCPSocketAction::default()
                });

                Some(shared_probe)
            }
            _ =>  None,
        }
    } else {
        None
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
                    requests: Some(BTreeMap::from([
                        ("memory".to_owned(), Quantity(spec.memory_request.clone())),
                        ("cpu".to_owned(), Quantity(spec.cpu_request.clone())),
                    ])),
                    limits: Some(BTreeMap::from([
                        ("memory".to_owned(), Quantity(spec.memory_limit.clone())),
                        ("cpu".to_owned(), Quantity(spec.cpu_limit.clone())),
                    ])),
                    ..ResourceRequirements::default()
                }),
                ..Container::default()
            };

            // configure available probes.
            container.liveness_probe = transform_probe(&spec.liveness_probe);
            container.readiness_probe = transform_probe(&spec.readiness_probe);
            container.startup_probe = transform_probe(&spec.startup_probe);

            // return container.
            container
        })
        .collect();

    containers
}

/**
 * Reconciles the deployment of a Nimble instance.
 *
 * This function orchestrates the deployment of a Nimble instance based on the provided context data.
 * It creates or updates a Kubernetes Deployment object with the specified configuration.
 *
 * # Arguments
 * - `nimble`: An Arc reference to the Nimble instance to reconcile.
 * - `ctx`: An Arc reference to the context data needed for reconciliation.
 *
 * # Returns
 * An Ok(Action) containing the requeue action with a specified duration on successful reconciliation,
 * or an Err(Error) if the reconciliation process encounters any errors.
 *
 * # Errors
 * - Returns an Error::MissingObjectKey if required object keys are missing.
 * - Returns an Error::NimbleObjectCreationFailed if the creation or update of the Nimble object fails.
 */
pub async fn reconcile(nimble: Arc<Nimble>, ctx: Arc<ContextData>) -> Result<Action, Error> {
    let client = &ctx.client;

    let oref = nimble.controller_owner_ref(&()).unwrap();

    let labels = &nimble.spec.deployment.labels;

    let containers = transform_containers(nimble.spec.deployment.containers.clone());

    let deployment: Deployment = Deployment {
        metadata: ObjectMeta {
            name: nimble.metadata.name.clone(),
            owner_references: Some(vec![oref]),
            annotations: nimble.spec.deployment.annotations.clone(),
            ..ObjectMeta::default()
        },
        spec: Some(DeploymentSpec {
            replicas: Some(nimble.spec.deployment.replicas),
            selector: LabelSelector {
                match_expressions: None,
                match_labels: Some(labels.clone()),
            },
            template: PodTemplateSpec {
                spec: Some(PodSpec {
                    containers,
                    ..PodSpec::default()
                }),
                metadata: Some(ObjectMeta {
                    labels: Some(labels.clone()),
                    annotations: nimble.spec.deployment.annotations.clone(),
                    ..ObjectMeta::default()
                }),
            },
            ..DeploymentSpec::default()
        }),
        ..Deployment::default()
    };

    let deployment_api = Api::<Deployment>::namespaced(
        client.clone(),
        nimble
            .metadata
            .namespace
            .as_ref()
            .ok_or_else(|| Error::MissingObjectKey(".metadata.namespace"))?,
    );

    deployment_api
        .patch(
            deployment
                .metadata
                .name
                .as_ref()
                .ok_or_else(|| Error::MissingObjectKey(".metadata.name"))?,
            &PatchParams::apply("nimble.ivaltryek.github.com"),
            &Patch::Apply(&deployment),
        )
        .await
        .map_err(Error::NimbleObjectCreationFailed)?;

    Ok(Action::requeue(Duration::from_secs(30)))
}

/**
 * Starts the main loop for the Nimble Deployment controller.
 *
 * This function initiates the main event loop for the Nimble controller, responsible for monitoring and reconciling Nimble resources in the Kubernetes cluster.
 *
 * Args:
 * - crd_api (Api<Nimble>): Reference to the Kubernetes API client for Nimble resources.
 * - context (Arc<ContextData>): Reference-counted handle to the controller context data.
 *
 * Returns:
 * - Future: Represents the completion of the controller loop.
 *
 * Process:
 * 1. Creates a new controller instance using the provided API client and default configuration.
 * 2. Configures the controller to shut down gracefully on receiving specific signals.
 * 3. Starts the controller loop, running the `reconcile` function for each Nimble resource change it detects.
 * 4. Within the loop, handles reconciliation results:
 *   - On success: logs a message with resource information.
 *   - On error: logs an error message with details.
 * 5. Waits for the loop to complete.
 */
pub async fn run_dp_controller(crd_api: Api<Nimble>, context: Arc<ContextData>) {
    Controller::new(crd_api.clone(), Config::default())
        .shutdown_on_signal()
        .run(reconcile, error_policy, context)
        .for_each(|reconcilation_result| async move {
            match reconcilation_result {
                Ok((nimble_resource, _)) => {
                    info!(msg = "Deployment reconciliation successful.",
                    resource_name = ?nimble_resource.name,
                    namespace = ?nimble_resource.namespace.unwrap(),
                    );
                }
                Err(reconciliation_err) => {
                    error!("Deployment reconciliation error: {:?}", reconciliation_err)
                }
            }
        })
        .await;
}

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use k8s_openapi::api::autoscaling::v2::{
    CrossVersionObjectReference, HorizontalPodAutoscaler, HorizontalPodAutoscalerSpec,
};
use kube::{
    api::{ObjectMeta, Patch, PatchParams},
    runtime::{controller::Action, watcher::Config, Controller},
    Api, Resource,
};
use tracing::{error, info};

use crate::{
    common::client::{error_policy, ContextData, Error},
    crds::nimble::Nimble,
    transformers::hpa::transform_metrics,
};

use futures::StreamExt;
use tokio::time::Duration;

static DOES_HPA_EXIST: AtomicBool = AtomicBool::new(false);

/**
 * Reconciles the HPA of a Nimble instance.
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
    match nimble.spec.hpa.clone() {
        Some(hpa_spec) => {
            let client = &ctx.client;

            let oref = nimble.controller_owner_ref(&()).unwrap();

            let hpa: HorizontalPodAutoscaler = HorizontalPodAutoscaler {
                metadata: ObjectMeta {
                    annotations: hpa_spec.annotations.clone(),
                    owner_references: Some(vec![oref]),
                    name: nimble.metadata.name.clone(),
                    ..ObjectMeta::default()
                },
                spec: Some(HorizontalPodAutoscalerSpec {
                    max_replicas: hpa_spec.max,
                    min_replicas: hpa_spec.min,
                    scale_target_ref: CrossVersionObjectReference {
                        api_version: Some("apps/v1".to_owned()),
                        kind: "Deployment".to_owned(),
                        name: nimble.metadata.name.clone().unwrap(),
                    },
                    metrics: transform_metrics(nimble.spec.hpa.clone()),
                    ..HorizontalPodAutoscalerSpec::default()
                }),
                ..HorizontalPodAutoscaler::default()
            };

            let hpa_api = Api::<HorizontalPodAutoscaler>::namespaced(
                client.clone(),
                nimble
                    .metadata
                    .namespace
                    .as_ref()
                    .ok_or_else(|| Error::MissingObjectKey(".metadata.namespace"))?,
            );

            hpa_api
                .patch(
                    hpa.metadata
                        .name
                        .as_ref()
                        .ok_or_else(|| Error::MissingObjectKey(".metadata.name"))?,
                    &PatchParams::apply("nimble.ivaltryek.github.com"),
                    &Patch::Apply(&hpa),
                )
                .await
                .map_err(Error::NimbleObjectCreationFailed)?;

            DOES_HPA_EXIST.store(true, Ordering::Relaxed);

            Ok(Action::requeue(Duration::from_secs(30)))
        }
        _ => {
            DOES_HPA_EXIST.store(false, Ordering::Relaxed);
            Ok(Action::await_change())
        }
    }
}

/**
 * Starts the main loop for the Nimble HPA controller.
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
pub async fn run_hpa_controller(crd_api: Api<Nimble>, context: Arc<ContextData>) {
    Controller::new(crd_api.clone(), Config::default())
        .shutdown_on_signal()
        .run(reconcile, error_policy, context)
        .for_each(|reconcilation_result| async move {
            match reconcilation_result {
                Ok((nimble_resource, _)) => {
                    // Log the reconciliation message only if service field exist in object manifest.
                    if DOES_HPA_EXIST.load(Ordering::Relaxed) {
                        info!(msg = "HPA reconciliation successful.",
                        resource_name = ?nimble_resource.name,
                        namespace = ?nimble_resource.namespace.unwrap(),
                        );
                    }
                }
                Err(reconciliation_err) => {
                    error!("HPA reconciliation error: {:?}", reconciliation_err)
                }
            }
        })
        .await;
}

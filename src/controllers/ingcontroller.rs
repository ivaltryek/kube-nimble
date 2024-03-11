use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use k8s_openapi::api::networking::v1::{Ingress, IngressSpec};
use kube::{
    api::{ObjectMeta, Patch, PatchParams},
    runtime::{controller::Action, watcher::Config, Controller},
    Api, Resource,
};
use tracing::{error, info};

use crate::{
    common::client::{error_policy, ContextData, Error},
    crds::nimble::Nimble,
    transformers::ingress::transform_rules,
};

use tokio::time::Duration;

use futures::StreamExt;

static DOES_ING_EXIST: AtomicBool = AtomicBool::new(false);

/**
 * Reconciles the Ingress of a Nimble instance.
 *
 * This function orchestrates the deployment of a Nimble instance based on the provided context data.
 * It creates or updates a Kubernetes Service object with the specified configuration.
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
    match nimble.spec.ingress.clone() {
        // Execution will go to this block only if ingress is mentioned in the object manifest.
        Some(ing) => {
            let client = &ctx.client;

            let oref = nimble.controller_owner_ref(&()).unwrap();

            let ingress: Ingress = Ingress {
                metadata: ObjectMeta {
                    annotations: ing.annotations,
                    owner_references: Some(vec![oref]),
                    name: nimble.metadata.name.clone(),
                    ..ObjectMeta::default()
                },
                spec: Some(IngressSpec {
                    ingress_class_name: ing.class,
                    rules: transform_rules(
                        ing.rules.clone(),
                        nimble.metadata.name.clone().unwrap(),
                    ),
                    ..IngressSpec::default()
                }),
                ..Ingress::default()
            };

            let ingress_api = Api::<Ingress>::namespaced(
                client.clone(),
                nimble
                    .metadata
                    .namespace
                    .as_ref()
                    .ok_or_else(|| Error::MissingObjectKey(".metadata.namespace"))?,
            );

            ingress_api
                .patch(
                    ingress
                        .metadata
                        .name
                        .as_ref()
                        .ok_or_else(|| Error::MissingObjectKey(".metadata.name"))?,
                    &PatchParams::apply("nimble.ivaltryek.github.com"),
                    &Patch::Apply(&ingress),
                )
                .await
                .map_err(Error::NimbleObjectCreationFailed)?;

            // Set the flag to true, since ingress is passed in object manifest.
            DOES_ING_EXIST.store(true, Ordering::Relaxed);

            Ok(Action::requeue(Duration::from_secs(30)))
        }
        _ => {
            // Set the flag to false, since ingress is not passed in object manifest.
            DOES_ING_EXIST.store(false, Ordering::Relaxed);
            Ok(Action::await_change())
        }
    }
}

/**
 * Starts the main loop for the Nimble ingress controller.
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
pub async fn run_ing_controller(crd_api: Api<Nimble>, context: Arc<ContextData>) {
    Controller::new(crd_api.clone(), Config::default())
        .shutdown_on_signal()
        .run(reconcile, error_policy, context)
        .for_each(|reconcilation_result| async move {
            match reconcilation_result {
                Ok((nimble_resource, _)) => {
                    // Log the reconciliation message only if ingress field exist in object manifest.
                    if DOES_ING_EXIST.load(Ordering::Relaxed) {
                        info!(msg = "Ingress reconciliation successful.",
                        resource_name = ?nimble_resource.name,
                        namespace = ?nimble_resource.namespace.unwrap(),
                        );
                    }
                }
                Err(reconciliation_err) => {
                    error!("Service reconciliation error: {:?}", reconciliation_err)
                }
            }
        })
        .await;
}

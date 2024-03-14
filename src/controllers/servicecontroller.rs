use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use k8s_openapi::api::core::v1::Service;
use kube::{
    api::{Patch, PatchParams},
    runtime::{controller::Action, watcher::Config, Controller},
    Api,
};
use tracing::{error, info};

use crate::{
    common::{
        client::{error_policy, ContextData, Error},
        helper::string_to_bool,
    },
    crds::nimble::Nimble,
    transformers::service::transform_svc,
};

use tokio::time::Duration;

use futures::StreamExt;

// Flag to print logs; if service does exist then print the info log in controller function
// else; don't print it. This creates confusion since service field optional and it might happen
// that even if somebody did not pass the field to manifest and controller will print
// reconciliation message.
static DOES_SVC_EXIST: AtomicBool = AtomicBool::new(false);

/**
 * Reconciles the deployment of a Nimble instance.
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
    match nimble.spec.service.clone() {
        // Execution will go to this block only if service is mentioned in the object manifest.
        Some(_svc) => {
            let client = &ctx.client;
            let is_dry_run = string_to_bool(std::env::var("DRY_RUN").unwrap_or("false".to_owned()));

            let service = transform_svc(nimble.clone(), is_dry_run);
            let service_api = Api::<Service>::namespaced(
                client.clone(),
                nimble
                    .metadata
                    .namespace
                    .as_ref()
                    .ok_or_else(|| Error::MissingObjectKey(".metadata.namespace"))?,
            );

            if is_dry_run {
                let _ = PatchParams {
                    dry_run: true,
                    ..PatchParams::default()
                };
                let params = PatchParams::apply("nimble.ivaltryek.github.com");
                let patch = Patch::Apply(&service);
                match service_api
                    .patch(nimble.metadata.name.as_ref().unwrap(), &params, &patch)
                    .await
                {
                    Ok(mut service) => {
                        // Set None to unnecessary fields for brevity.
                        service.metadata.managed_fields = None;
                        service.status = None;
                        let yaml = serde_yaml::to_string(&service).unwrap();
                        println!("---\n# service.yaml\n\n{}", yaml);
                    }
                    Err(e) => {
                        error!("{:?}", e);
                    }
                }
                return Ok(Action::await_change());
            }

            service_api
                .patch(
                    service
                        .metadata
                        .name
                        .as_ref()
                        .ok_or_else(|| Error::MissingObjectKey(".metadata.name"))?,
                    &PatchParams::apply("nimble.ivaltryek.github.com"),
                    &Patch::Apply(&service),
                )
                .await
                .map_err(Error::NimbleObjectCreationFailed)?;

            // Set the flag to true, since service is passed in object manifest.
            DOES_SVC_EXIST.store(true, Ordering::Relaxed);

            Ok(Action::requeue(Duration::from_secs(30)))
        }
        _ => {
            // Set the flag to false, since service is not passed in object manifest.
            DOES_SVC_EXIST.store(false, Ordering::Relaxed);
            Ok(Action::await_change())
        }
    }
}

/**
 * Starts the main loop for the Nimble service controller.
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
#[allow(dead_code)]
pub async fn run_svc_controller(crd_api: Api<Nimble>, context: Arc<ContextData>) {
    Controller::new(crd_api.clone(), Config::default())
        .shutdown_on_signal()
        .run(reconcile, error_policy, context)
        .for_each(|reconcilation_result| async move {
            match reconcilation_result {
                Ok((nimble_resource, _)) => {
                    // Log the reconciliation message only if service field exist in object manifest.
                    if DOES_SVC_EXIST.load(Ordering::Relaxed) {
                        info!(msg = "Service reconciliation successful.",
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

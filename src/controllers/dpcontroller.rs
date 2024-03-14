use std::sync::Arc;

use k8s_openapi::{api::apps::v1::Deployment, Metadata};
use kube::{
    api::{Patch, PatchParams},
    runtime::{controller::Action, watcher::Config, Controller},
    Api,
};

use crate::{
    common::helper::string_to_bool, crds::nimble::Nimble,
    transformers::deployment::transform_deployment,
};

use crate::common::client::{error_policy, ContextData, Error};

use tokio::time::Duration;

use futures::StreamExt;

use tracing::{error, info};

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
    // setting up env for dry_run usecase.
    let is_dry_run = string_to_bool(std::env::var("DRY_RUN").unwrap_or("false".to_owned()));
    let client = &ctx.client;

    let deployment: Deployment = transform_deployment(nimble.clone(), is_dry_run);

    let deployment_api = Api::<Deployment>::namespaced(
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
        let patch = Patch::Apply(&deployment);
        match deployment_api
            .patch(
                deployment.metadata().name.as_ref().unwrap(),
                &params,
                &patch,
            )
            .await
        {
            Ok(mut dp) => {
                // Set None to unnecessary fields for brevity.
                dp.metadata.managed_fields = None;
                dp.status = None;
                let yaml = serde_yaml::to_string(&dp).unwrap();
                println!("---\n# deployment.yaml\n\n{}", yaml);
            }
            Err(e) => {
                error!("{:?}", e);
            }
        }
        return Ok(Action::await_change());
    }

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
#[allow(dead_code)]
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

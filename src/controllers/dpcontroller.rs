use std::{collections::BTreeMap, sync::Arc};

use k8s_openapi::{
    api::{
        apps::v1::{Deployment, DeploymentSpec},
        core::v1::{Container, PodSpec, PodTemplateSpec},
    },
    apimachinery::pkg::apis::meta::v1::LabelSelector,
};
use kube::{
    api::{ObjectMeta, Patch, PatchParams},
    runtime::{controller::Action, watcher::Config, Controller},
    Api, Resource,
};

use crate::crds::nimble::Nimble;

use super::client::{error_policy, ContextData, Error};

use tokio::time::Duration;

use futures::StreamExt;

use tracing::{error, info};

pub async fn reconcile(nimble: Arc<Nimble>, ctx: Arc<ContextData>) -> Result<Action, Error> {
    let client = &ctx.client;

    let oref = nimble.controller_owner_ref(&()).unwrap();
    let mut labels: BTreeMap<String, String> = BTreeMap::new();

    // TODO: make labels dynamic.
    labels.insert("app".to_owned(), "default".to_owned());

    let deployment: Deployment = Deployment {
        metadata: ObjectMeta {
            name: nimble.metadata.name.clone(),
            owner_references: Some(vec![oref]),
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
                    containers: vec![Container {
                        // TODO: change container name to dynamic!
                        name: "default".to_owned(),
                        image: Some(nimble.spec.deployment.image.clone()),
                        ..Container::default()
                    }],
                    ..PodSpec::default()
                }),
                metadata: Some(ObjectMeta {
                    labels: Some(labels.clone()),
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

pub async fn run_dp_controller(crd_api: Api<Nimble>, context: Arc<ContextData>) {
    Controller::new(crd_api.clone(), Config::default())
        .shutdown_on_signal()
        .run(reconcile, error_policy, context)
        .for_each(|reconcilation_result| async move {
            match reconcilation_result {
                Ok(nimble_resource) => {
                    info!("Reconciliation successful. Resource: {:?}", nimble_resource);
                }
                Err(reconciliation_err) => {
                    error!("Reconciliation error: {:?}", reconciliation_err)
                }
            }
        })
        .await;
}

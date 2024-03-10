use std::sync::Arc;
mod common;
mod controllers;
mod crds;
mod transformers;
use kube::{Api, Client};
use tracing::info;

use crate::common::client::ContextData;
use crate::controllers::dpcontroller::run_dp_controller;
use crate::controllers::hpacontroller::run_hpa_controller;
use crate::controllers::servicecontroller::run_svc_controller;
use crate::crds::nimble::Nimble;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let kubernetes_client: Client = Client::try_default()
        .await
        .expect("Couldn't find KUBECONFIG Variable");

    let crd_api = Api::<Nimble>::all(kubernetes_client.clone());
    let context: Arc<ContextData> = Arc::new(ContextData::new(kubernetes_client.clone()));

    info!("starting nimble controller");

    let (_, _, _) = futures::join!(
        run_dp_controller(crd_api.clone(), context.clone()),
        run_svc_controller(crd_api.clone(), context.clone()),
        run_hpa_controller(crd_api.clone(), context.clone())
    );

    info!("controller terminated");
}

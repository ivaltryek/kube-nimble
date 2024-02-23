use std::sync::Arc;
mod controllers;
mod crds;
use kube::{Api, Client};
use tracing::info;

use crate::{
    controllers::{client::ContextData, dpcontroller::run_dp_controller},
    crds::nimble::Nimble,
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let kubernetes_client: Client = Client::try_default()
        .await
        .expect("Couldn't find KUBECONFIG Variable");

    let crd_api = Api::<Nimble>::all(kubernetes_client.clone());
    let context: Arc<ContextData> = Arc::new(ContextData::new(kubernetes_client.clone()));

    info!("starting nimble controller");

    run_dp_controller(crd_api.clone(), context).await;

    info!("controller terminated");
}

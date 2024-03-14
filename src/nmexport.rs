use std::sync::Arc;

use clap::Parser;
use kube::Client;

use crate::{common::client::ContextData, crds::nimble::Nimble};

mod common;
mod controllers;
mod crds;
mod transformers;

#[derive(Parser, Debug)]
#[command(
    author = "Meet Vasani",
    version = "0.7.2",
    about = "dry-run kube-nimble resource"
)]
pub struct Args {
    #[arg(long = "resource")]
    pub resource_path: String,
}

impl Args {
    pub fn new(resource: String) -> Self {
        Self {
            resource_path: resource,
        }
    }

    pub fn load_resource_yaml(&self) -> Nimble {
        let file_handler =
            std::fs::File::open(&self.resource_path).expect("Could not load find the file");
        let file_data: Nimble = serde_yaml::from_reader(file_handler).unwrap();
        file_data
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let kubernetes_client: Client = Client::try_default()
        .await
        .expect("Couldn't find KUBECONFIG Variable");

    let context: Arc<ContextData> = Arc::new(ContextData::new(kubernetes_client.clone()));

    std::env::set_var("DRY_RUN", "TRUE");

    let input: Args = Args::parse();

    let arg = Args::new(input.resource_path);

    let nimble_object = arg.load_resource_yaml();

    let _ =
        crate::controllers::dpcontroller::reconcile(nimble_object.clone().into(), context.clone())
            .await;

    let _ =
        crate::controllers::hpacontroller::reconcile(nimble_object.clone().into(), context.clone())
            .await;

    let _ = crate::controllers::servicecontroller::reconcile(
        nimble_object.clone().into(),
        context.clone(),
    )
    .await;

    let _ =
        crate::controllers::ingcontroller::reconcile(nimble_object.clone().into(), context.clone())
            .await;
}

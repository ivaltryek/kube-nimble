// Generate CustomResourceDefinition from struct.
// Usage: cargo run --bin crdgen
use kube::CustomResourceExt;

mod crds;

fn main() {
    print!(
        "{}",
        serde_yaml::to_string(&crate::crds::nimble::Nimble::crd()).unwrap()
    )
}

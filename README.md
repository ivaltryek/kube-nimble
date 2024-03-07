# kube-nimble
  nimble /ˈnɪmbl/ - quick and light in movement or action; agile.

This project began from a place of curiosity about Kubernetes CRDs and their controllers, as well as a desire to learn Rust. I never anticipated how straightforward it would be to craft controller logic in Rust. It's evident that many individuals encounter challenges when managing numerous manifests for various environments (although tools like Helm and Kustomize address this). This tool offers a streamlined alternative for deploying native Kubernetes objects with minimal overhead, eliminating the need to create lengthy manifests for each Kubernetes object.

As it stands, this tool currently supports the minimal creation of deployments, offering arguments such as image and replica count. However, my plan is to extend its capabilities to encompass all other Kubernetes objects in the near future.

### Steps to run locally on linux
  - Install [Rust](https://www.rust-lang.org/tools/install)
  - Kubernetes Cluster (Local / Cloud)
  - export kubeconfig of cluster using `export KUBECONFIG=<path-to-config>`
  - Apply CRD to the cluster using `kubectl create -f crd/nimble.ivaltryek.github.com.yaml`
  - Build the project using `cargo build`
  - Run the operator using `cargo run`


If the operator initializes without encountering any errors, then you have successfully followed the steps.
  - Finally, Create the custom object from examples
    - This objects is created in namespace test; if it does not exist use `kubectl create ns test`
    - to apply the object: `kubectl create -f examples/simple-deployment.yaml`

## API Reference 
https://ivaltryek.github.io/kube-nimble/ <br>
[Raw MD Files Generated by CI](https://github.com/ivaltryek/kube-nimble/tree/gh-pages/docs)

## Development
   To develop this project locally, just follow the steps that is mentioned in [run locally on linux](#Steps-to-run-locally-on-linux)

   ### Generate CRD Manifest
   ```
   cargo run --bin crdgen > crd.yaml
   ```
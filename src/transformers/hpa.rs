use std::sync::Arc;

use k8s_openapi::api::autoscaling::v2::{
    CrossVersionObjectReference, HorizontalPodAutoscaler, HorizontalPodAutoscalerSpec, MetricSpec,
    MetricTarget, ResourceMetricSource,
};
use kube::{api::ObjectMeta, Resource};

use crate::crds::{hpaspec::HPASpec, nimble::Nimble};

fn transform_metrics(hpa_spec: Option<HPASpec>) -> Option<Vec<MetricSpec>> {
    let mut metric_spec_vec = Vec::new();

    match hpa_spec.clone() {
        Some(hpa) => {
            if let Some(resource_policy) = hpa.resource_policy {
                metric_spec_vec.push(MetricSpec {
                    type_: "Resource".to_owned(),
                    resource: Some(ResourceMetricSource {
                        name: resource_policy.name,
                        target: MetricTarget {
                            average_utilization: resource_policy.average_utilization,
                            type_: resource_policy.type_,
                            ..MetricTarget::default()
                        },
                    }),
                    ..MetricSpec::default()
                });
            }
            Some(metric_spec_vec)
        }
        _ => None,
    }
}

pub fn transform_hpa(nimble: Arc<Nimble>, is_dry_run: bool) -> HorizontalPodAutoscaler {
    let hpa_spec = nimble.spec.hpa.clone().unwrap();
    let hpa: HorizontalPodAutoscaler = HorizontalPodAutoscaler {
        metadata: if is_dry_run {
            ObjectMeta {
                name: nimble.metadata.name.clone(),
                annotations: hpa_spec.annotations.clone(),
                ..ObjectMeta::default()
            }
        } else {
            let oref = nimble.controller_owner_ref(&()).unwrap();
            ObjectMeta {
                name: nimble.metadata.name.clone(),
                owner_references: Some(vec![oref]),
                annotations: hpa_spec.annotations.clone(),
                ..ObjectMeta::default()
            }
        },
        spec: Some(HorizontalPodAutoscalerSpec {
            max_replicas: hpa_spec.max,
            min_replicas: hpa_spec.min,
            scale_target_ref: CrossVersionObjectReference {
                api_version: Some("apps/v1".to_owned()),
                kind: "Deployment".to_owned(),
                name: nimble.metadata.name.clone().unwrap(),
            },
            metrics: transform_metrics(Some(hpa_spec)),
            ..HorizontalPodAutoscalerSpec::default()
        }),
        ..HorizontalPodAutoscaler::default()
    };
    hpa
}

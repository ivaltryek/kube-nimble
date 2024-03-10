use k8s_openapi::api::autoscaling::v2::{MetricSpec, MetricTarget, ResourceMetricSource};

use crate::crds::hpaspec::HPASpec;

pub fn transform_metrics(hpa_spec: Option<HPASpec>) -> Option<Vec<MetricSpec>> {
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

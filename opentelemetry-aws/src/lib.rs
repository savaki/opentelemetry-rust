//! # OpenTelemetry AWS Exporter
//!
//! Collects OpenTelemetry spans and reports them to AWS X-Ray.
//! See [AWS X-Ray](https://aws.amazon.com/xray/) for additional details.
//!
//! ### AWS collector example
//!
//! This example expects AWS credentials are present in the environment:
//!
//! ```rust,no_run
//! use opentelemetry::{api::Key, global, sdk};
//! use opentelemetry_aws::ExporterConfig;
//! use std::net::{SocketAddr, IpAddr, Ipv4Addr};
//!
//! fn init_tracer() {
//!     let exporter = opentelemetry_aws::Exporter::from_config(
//!        ExporterConfig::builder()
//!            .with_service_name("opentelemetry-backend".to_owned())
//!            .build());
//!     let provider = sdk::Provider::builder()
//!         .build();
//!
//!     global::set_provider(provider);
//! }
//! ```
//!
#![deny(missing_docs, unreachable_pub, missing_debug_implementations)]
#![cfg_attr(test, deny(warnings))]

pub mod id;
pub(crate) mod format;
pub mod propagation;

/// neat
mod service;

#[macro_use]
extern crate typed_builder;

#[macro_use]
extern crate lazy_static;

use rusoto_core::{Region};
use rusoto_xray::{XRayClient, XRay};
use futures::executor::block_on;
use opentelemetry::exporter::trace;
use std::fmt::{Debug, Formatter, Result};
use std::sync::Arc;
use std::vec::{Vec};

/// AWS x-ray exporter
pub struct Exporter {
    config: ExporterConfig
}

/// AWS-specific configuration used to initialize the `Exporter`.
#[derive(Clone)]
pub struct ExporterConfig {
    client: XRayClient,
    service_name: String,
}

impl Debug for ExporterConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("ExporterConfig")
            .field("service_name", &self.service_name).
            finish()
    }
}

/// Builder for `ExporterConfig` struct.
#[derive(Debug)]
pub struct ExporterConfigBuilder {
    service_name: Option<String>,
}

impl Default for ExporterConfigBuilder {
    /// initialize ExporterConfigBuilder
    fn default() -> Self {
        ExporterConfigBuilder {
            service_name: None,
        }
    }
}

impl ExporterConfig {
    /// Create an export config builder
    pub fn builder() -> ExporterConfigBuilder {
        ExporterConfigBuilder::default()
    }
}

impl ExporterConfigBuilder {
    /// Create `ExporterConfig` struct from current `ExporterConfigBuilder`
    pub fn build(&self) -> ExporterConfig {
        let service_name = self
            .service_name
            .clone()
            .unwrap_or_else(|| "DEFAULT".to_owned());
        let client = XRayClient::new(Region::default());

        ExporterConfig {
            client,
            service_name,
        }
    }


    /// Assign the service name for `ConfigBuilder`
    pub fn with_service_name(&mut self, name: String) -> &mut Self {
        self.service_name = Some(name);
        self
    }
}

impl Exporter {
    /// Creates new `Exporter` from a given `ExporterConfig`.
    pub fn from_config(config: ExporterConfig) -> Self {
        Exporter {
            config: config.clone(),
        }
    }
}

impl Debug for Exporter {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("Exporter")
            .finish()
    }
}

impl trace::SpanExporter for Exporter {
    fn export(&self, batch: Vec<Arc<trace::SpanData>>) -> trace::ExportResult {
        let trace_segment_documents = to_segments(batch);
        let req = rusoto_xray::PutTraceSegmentsRequest {
            trace_segment_documents,
        };
        match block_on(self.config.client.put_trace_segments(req)) {
            Ok(_res) => trace::ExportResult::Success,
            Err(_res) => trace::ExportResult::FailedNotRetryable,
        }
    }

    fn shutdown(&self) {}
}

// converts common opentelemetry SpanData with aws platform specific segments
fn to_segments(batch: Vec<Arc<trace::SpanData>>) -> Vec<String> {
    batch.iter().map(|_data|
        serde_json::to_string(&service::xray::Segment::builder().build()).unwrap()
    ).collect()
}

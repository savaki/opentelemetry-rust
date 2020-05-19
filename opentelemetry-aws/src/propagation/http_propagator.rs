//! # OpenTelemetry AWS Exporter
//!
use opentelemetry::api;
use opentelemetry::api::{Context, Carrier, TraceContextExt};
use crate::format;

const HEADER: &'static str = "X-Amzn-Trace-Id";

/// asalksjdlask
#[derive(Debug, Default)]
pub struct HttpPropagator {}

impl api::HttpTextFormat for HttpPropagator {
    fn inject_context(&self, context: &Context, carrier: &mut dyn Carrier) {
        let span_context = context.span().span_context();
        if !span_context.is_valid() {
            return;
        }

        let value = format::span_context(span_context);
        carrier.set(
            HEADER,
            value,
        );
    }

    fn extract_with_context(&self, _cx: &Context, _carrier: &dyn Carrier) -> Context {
        unimplemented!()
    }
}

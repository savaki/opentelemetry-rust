use hyper::{body::Body, Client};
use opentelemetry::api::{Context, HttpTextFormat, TraceContextExt, Tracer};
use opentelemetry::{api, global, sdk};

struct ClientHeaderMapCarrier<'a>(&'a mut hyper::header::HeaderMap);

impl<'a> api::Carrier for ClientHeaderMapCarrier<'a> {
    fn get(&self, key: &'static str) -> Option<&str> {
        self.0.get(key).and_then(|value| value.to_str().ok())
    }

    fn set(&mut self, key: &'static str, value: String) {
        self.0.insert(key, value.parse().unwrap());
    }
}

fn init_tracer() {
    // Create stdout exporter to be able to retrieve the collected spans.
    let exporter = opentelemetry_aws::Exporter::from_config(
        opentelemetry_aws::ExporterConfig::builder().
            with_service_name("trace-demo".to_owned()).
            build(),
    );

    // For the demonstration, use `Sampler::Always` sampler to sample all traces. In a production
    // application, use `Sampler::Parent` or `Sampler::Probability` with a desired probability.
    let provider = sdk::Provider::builder()
        .with_simple_exporter(exporter)
        .with_config(sdk::Config {
            default_sampler: Box::new(sdk::Sampler::Always),
            id_generator: Box::new(opentelemetry_aws::id::Generator::default()),
            ..Default::default()
        })
        .build();

    global::set_provider(provider);
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    init_tracer();

    let client = Client::new();
    let propagator = api::TraceContextPropagator::new();
    let span = global::tracer("example/client").start("say hello");
    let cx = Context::current_with_span(span);

    let mut req = hyper::Request::builder().uri("http://127.0.0.1:3000");
    propagator.inject_context(&cx, &mut ClientHeaderMapCarrier(req.headers_mut().unwrap()));
    let res = client.request(req.body(Body::from("Hallo!"))?).await?;

    cx.span().add_event(
        "Got response!".to_string(),
        vec![api::KeyValue::new("status", res.status().to_string())],
    );

    Ok(())
}

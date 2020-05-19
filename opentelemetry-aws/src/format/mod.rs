use opentelemetry::api;
use regex::Regex;

/// an aws trace id consists of a 32 bit unix timestamp plus 96 bits of
/// id.  ROOT_MASK simplifies extracting the aws root id from an opentelemetry
/// trace_id
const ROOT_MASK: u128 = 0xffffffffffffffffffffffff;

// span_context returns a formatted xray span
pub(crate) fn span_context(span_context: api::SpanContext) -> String {
    let trace_id = span_context.trace_id().to_u128();
    format!(
        "Root=1-{:08x}-{:024x};Parent={:016x};Sampled={:b}",
        trace_id >> 96,
        trace_id & ROOT_MASK,
        span_context.span_id().to_u64(),
        if span_context.is_sampled() { 1 } else { 0 },
    )
}

pub(crate) fn parse_header(generator: Box<dyn api::IdGenerator>, header: &str) -> Option<api::SpanContext> {
    lazy_static! {
        static ref RE_ROOT: Regex = Regex::new(r"^Root=1-([[:xdigit:]]+)-([[:xdigit:]]+)").unwrap();
        static ref RE_PARENT: Regex = Regex::new(r"Parent=([[:xdigit:]]+)").unwrap();
        static ref RE_SAMPLED: Regex = Regex::new(r"Sampled=1").unwrap();
    }

    let trace_id = RE_ROOT.captures(header)
        .and_then(|cap| {
            match u32::from_str_radix(&cap[1], 16)
                .and_then(|timestamp| u128::from_str_radix(&cap[2], 16)
                    .and_then(|root_id| {
                        let trace_id = (timestamp as u128) << 96 | (root_id & ROOT_MASK);
                        Ok(api::TraceId::from_u128(trace_id))
                    })
                )
            {
                Err(_) => None,
                Ok(e) => Some(e),
            }
        })?;

    let parent_id = RE_PARENT.captures(header)
        .and_then(|cap| {
            match u64::from_str_radix(&cap[1], 16)
                .and_then(|id| Ok(api::SpanId::from_u64(id)))
            {
                Err(_) => None,
                Ok(e) => Some(e),
            }
        })
        .or_else(|| Some(generator.new_span_id()));

    let trace_flags = if RE_SAMPLED.is_match(header) { api::TRACE_FLAG_SAMPLED } else { api::TRACE_FLAGS_UNUSED };

    Some(api::SpanContext::new(
        trace_id,
        parent_id.unwrap(),
        trace_flags,
        true,
    ))
}

#[cfg(test)]
mod tests {
    use crate::format::parse_header;

    #[test]
    fn test_span_context() {
        let raw = [
            "Root=1-5759e988-bd862e3fe1be46a994272793;Parent=53995c3f42cd8ad8;Sampled=1",
            "Root=1-5759e988-bd862e3fe1be46a994272793;Parent=53995c3f42cd8ad8;Sampled=0",
        ];

        raw.iter().for_each(|want| {
            let span_context = parse_header(Box::new(crate::id::Generator::default()), want).unwrap();
            let got = crate::format::span_context(span_context);
            assert_eq!(got, *want);
        });
    }
}


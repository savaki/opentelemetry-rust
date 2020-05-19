//! eek
use opentelemetry::api;
use rand::{rngs, Rng};
use std::cell::RefCell;
use std::time::SystemTime;

/// Generates Trace and Span ids
#[derive(Clone, Debug, Default)]
pub struct Generator {
    _private: (),
}

impl api::IdGenerator for Generator {
    /// Generate new `TraceId` using thread local rng
    fn new_trace_id(&self) -> api::TraceId {
        let mut id: u128 = 0;
        CURRENT_RNG.with(|rng| id = rng.borrow_mut().gen());

        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        api::TraceId::from_u128((now as u128) << 12 | (id & 0xfff))
    }

    /// Generate new `SpanId` using thread local rng
    fn new_span_id(&self) -> api::SpanId {
        CURRENT_RNG.with(|rng| api::SpanId::from_u64(rng.borrow_mut().gen()))
    }
}

thread_local! {
    /// Store random number generator for each thread
    static CURRENT_RNG: RefCell<rngs::ThreadRng> = RefCell::new(rngs::ThreadRng::default());
}

#[cfg(test)]
mod tests {
    use opentelemetry::api::IdGenerator;
    use std::time::SystemTime;
    use crate::id::Generator;

    #[test]
    fn test_new_trace_id() {
        let trace_id = Generator::default().new_trace_id();
        let got = (trace_id.to_u128() >> 12) as u64;
        let want = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();

        assert!(want - got <= 1);
        assert!(got & 0xfff > 0);
    }
}

use hex;
use serde::{Serialize, Serializer};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::ser::SerializeSeq;

static SDK: &str = "opentelemetry_aws 1.2.3";

fn serialize_time<S: Serializer>(x: &SystemTime, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_u64(x.duration_since(UNIX_EPOCH).unwrap().as_secs())
}

fn serialize_trace_id<S: Serializer>(o: &Option<u128>, s: S) -> Result<S::Ok, S::Error> {
    match o {
        None => s.serialize_none(),
        Some(x) => s.serialize_str(&format!("1-{:08x}-{:012x}", x >> 12, x & 0x7ff))
    }
}

fn serialize_u64_to_hex<S: Serializer>(x: &u64, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&hex::encode(x.to_be_bytes()))
}

fn serialize_vec_u64_to_hex<S: Serializer>(o: &Option<Vec<u64>>, s: S) -> Result<S::Ok, S::Error> {
    match o {
        None => s.serialize_none(),
        Some(xx) => {
            let mut seq = s.serialize_seq(Some(xx.len()))?;
            for x in xx {
                seq.serialize_element(&hex::encode(x.to_be_bytes()))?;
            }
            seq.end()
        }
    }
}

fn serialize_opt_u64_to_hex<S: Serializer>(o: &Option<u64>, s: S) -> Result<S::Ok, S::Error> {
    match o {
        None => s.serialize_none(),
        Some(x) => serialize_u64_to_hex(x, s)
    }
}

pub(crate) enum Origin {
    EC2Instance,
    ECSContainer,
    ElasticBeanstalk,
}

impl Origin {
    fn to_str(&self) -> &str {
        match self {
            Origin::EC2Instance => "AWS::EC2::Instance",
            Origin::ECSContainer => "AWS::ECS::Container",
            Origin::ElasticBeanstalk => "AWS::ElasticBeanstalk::Environment",
        }
    }
}

impl Serialize for Origin {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        s.serialize_str(self.to_str())
    }
}

pub(crate) enum Value {
    String(String),
    Number(i64),
}

impl Serialize for Value {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        match self {
            Value::String(v) => s.serialize_str(v),
            Value::Number(v) => s.serialize_i64(*v),
        }
    }
}

/// Information about an Amazon ECS container.
#[derive(TypedBuilder, Serialize)]
pub(crate) struct AWSECS {
    /// The container ID of the container running your application.
    #[builder(default = None, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    container: Option<String>,
}

/// Information about an EC2 instance.
#[derive(TypedBuilder, Serialize)]
pub(crate) struct AWSEC2 {
    /// The Availability Zone in which the instance is running.
    #[builder(default = None, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    availability_zone: Option<String>,

    /// The instance ID of the EC2 instance.
    #[builder(default = None, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    instance_id: Option<String>,
}

/// Information about an Elastic Beanstalk environment.
#[derive(TypedBuilder, Serialize)]
pub(crate) struct AWSElasticBeanstalk {
    /// number indicating the ID of the last successful deployment
    /// to the instance that served the request.
    #[builder(default = None, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    deployment_id: Option<u64>,

    /// The name of the environment.
    #[builder(default = None, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    environment_name: Option<String>,

    /// The name of the application version that is currently
    /// deployed to the instance that served the request.
    #[builder(default = None, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    version_label: Option<String>,
}

/// Information about the sdk calling put segments
#[derive(TypedBuilder, Serialize)]
pub(crate) struct AWSXRay {
    /// defines this sdk publishing to aws
    #[builder(default = SDK)]
    sdk: &'static str,
}

/// Information about the resource on which your application is running.
#[derive(TypedBuilder, Serialize)]
pub(crate) struct AWS {
    // Segment

    /// If your application accesses resources in a different account,
    /// or sends segments to a different account, record the ID of the
    /// account that owns the AWS resource that your application accessed.
    #[builder(default = None, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    account_id: Option<String>,

    /// Information about an Amazon ECS container.
    #[builder(default = None, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    ecs: Option<AWSECS>,

    /// Information about an EC2 instance.
    #[builder(default = None, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    ec2: Option<AWSEC2>,

    /// formation about an Elastic Beanstalk environment.
    #[builder(default = None, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    elastic_beanstalk: Option<AWSElasticBeanstalk>,

    #[builder(default = AWSXRay::builder().build())]
    xray: AWSXRay,

    // Subsegments

    // The name of the API action invoked against an AWS
    // service or resource.
    #[builder(default = None, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    operation: Option<String>,

    /// If the resource is in a region different from your
    /// application, record the region. For example, us-west-2.
    #[builder(default = None, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    region: Option<String>,

    /// Unique identifier for the request.
    #[builder(default = None, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    request_id: Option<String>,

    /// For operations on an Amazon SQS queue, the queue's URL.
    #[builder(default = None, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    queue_url: Option<String>,

    /// For operations on a DynamoDB table, the name of the table.
    #[builder(default = None, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    table_name: Option<String>,
}

#[derive(TypedBuilder, Serialize)]
pub(crate) struct HttpRequest {
    #[builder(default = None, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    client_ip: Option<String>,

    #[builder(default = None, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    method: Option<String>,

    #[builder(default = None, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    traced: Option<bool>,

    #[builder(default = None, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,

    #[builder(default = None, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    user_agent: Option<String>,

    #[builder(default = None, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    x_forwarded_for: Option<bool>,
}

#[derive(TypedBuilder, Serialize)]
pub(crate) struct HttpResponse {
    #[builder(default = None, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    content_length: Option<i32>,

    #[builder(default = None, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<i8>,
}

#[derive(TypedBuilder, Serialize)]
pub(crate) struct Http {
    #[builder(default = None, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    request: Option<HttpRequest>,

    #[builder(default = None, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    response: Option<HttpResponse>,
}

/// Segment
/// as defined https://docs.aws.amazon.com/xray/latest/devguide/xray-api-segmentdocuments.html
#[derive(TypedBuilder, Serialize)]
pub(crate) struct Segment {
    /// key-value pairs that you want X-Ray to index for search.
    #[builder(default = None, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    annotations: Option<HashMap<String, Value>>,

    /// information about the downstream AWS resource that your application called.
    #[builder(default = None, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    aws: Option<AWS>,

    #[serde(serialize_with = "serialize_time")]
    #[builder(default = SystemTime::now())]
    end_time: SystemTime,

    /// information about an outgoing HTTP call.
    #[builder(default = None, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    http: Option<Http>,

    #[builder(default = 0)]
    #[serde(serialize_with = "serialize_u64_to_hex")]
    id: u64,

    #[builder(default = false)]
    is_progress: bool,

    /// object with any additional data that you want to store in the segment.
    #[builder(default = None, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<HashMap<String, Value>>,

    #[builder(default)]
    name: String,

    #[builder(default = None, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    origin: Option<Origin>,

    /// array of subsegment IDs that identifies subsegments
    /// with the same parent that completed prior to this subsegment.
    #[builder(default = None, setter(strip_option))]
    #[serde(serialize_with = "serialize_vec_u64_to_hex", skip_serializing_if = "Option::is_none")]
    precursor_ids: Option<Vec<u64>>,

    #[builder(default = None, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    service: Option<String>,

    #[builder(default = None, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    user: Option<String>,

    #[builder(default = None, setter(strip_option))]
    #[serde(serialize_with = "serialize_opt_u64_to_hex")]
    parent_id: Option<u64>,

    #[builder(default = SystemTime::now())]
    #[serde(serialize_with = "serialize_time")]
    start_time: SystemTime,

    /// array of subsegment objects.
    #[builder(default = vec ! [])]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    subsegments: Vec<Segment>,

    #[builder(default = None, setter(strip_option))]
    #[serde(serialize_with = "serialize_trace_id", skip_serializing_if = "Option::is_none")]
    trace_id: Option<u128>,
}

#[cfg(test)]
mod tests {
    use crate::service::xray::{Segment, Value};
    use crate::service::xray::AWS;
    use crate::service::xray::Origin::{ECSContainer, EC2Instance, ElasticBeanstalk};
    use std::time::{Duration, SystemTime};
    use std::ops::Add;
    use std::collections::HashMap;
    use crate::service::xray::Value::Number;

    #[test]
    fn test_empty() {
        let segment = Segment::builder()
            .aws(AWS::builder().build())
            .name(String::from("the name"))
            .id(123)
            .trace_id(456 << 12)
            .service(String::from("eek"))
            .origin(ECSContainer)
            .parent_id(789)
            .start_time(SystemTime::UNIX_EPOCH.add(Duration::new(1, 0)))
            .end_time(SystemTime::UNIX_EPOCH.add(Duration::new(2, 0)))
            .subsegments(vec![
                Segment::builder()
                    .name(String::from("child name"))
                    .start_time(SystemTime::UNIX_EPOCH.add(Duration::new(3, 0)))
                    .end_time(SystemTime::UNIX_EPOCH.add(Duration::new(4, 0)))
                    .build(),
            ])
            .build();
        test_json_serialization(segment, r#"{"aws":{"xray":{"sdk":"opentelemetry_aws 1.2.3"}},"end_time":2,"id":"000000000000007b","is_progress":false,"name":"the name","origin":"AWS::ECS::Container","service":"eek","parent_id":"0000000000000315","start_time":1,"subsegments":[{"end_time":4,"id":"0000000000000000","is_progress":false,"name":"child name","parent_id":null,"start_time":3}],"trace_id":"1-000001c8-000000000000"}"#);
    }

    fn test_json_serialization(content: Segment, desired: &str) {
        let result = serde_json::to_string(&content).unwrap();
        assert_eq!(result, desired.to_owned());
    }

    #[test]
    fn test_origin_ec2() {
        let segment =Segment::builder()
            .origin(EC2Instance)
            .start_time(SystemTime::UNIX_EPOCH.add(Duration::new(1, 0)))
            .end_time(SystemTime::UNIX_EPOCH.add(Duration::new(2, 0)))
            .build();
        test_json_serialization(segment, r#"{"end_time":2,"id":"0000000000000000","is_progress":false,"name":"","origin":"AWS::EC2::Instance","parent_id":null,"start_time":1}"#)
    }

    #[test]
    fn test_origin_elastic_beanstalk() {
        let segment =Segment::builder()
            .origin(ElasticBeanstalk)
            .start_time(SystemTime::UNIX_EPOCH.add(Duration::new(1, 0)))
            .end_time(SystemTime::UNIX_EPOCH.add(Duration::new(2, 0)))
            .build();
        test_json_serialization(segment, r#"{"end_time":2,"id":"0000000000000000","is_progress":false,"name":"","origin":"AWS::ElasticBeanstalk::Environment","parent_id":null,"start_time":1}"#)
    }

    #[test]
    fn test_value_string() {
        let mut m = HashMap::new();
        m.insert("str", Value::String(String::from("s")));
        let got = serde_json::to_string(&m);
        assert_eq!(got.unwrap(), r#"{"str":"s"}"#);
    }

    #[test]
    fn test_value_number() {
        let mut m = HashMap::new();
        m.insert("num", Number(123));
        let got = serde_json::to_string(&m);
        assert_eq!(got.unwrap(), r#"{"num":123}"#);
    }
}

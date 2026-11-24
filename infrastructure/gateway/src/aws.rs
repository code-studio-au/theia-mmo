use crate::{Instance, Realm, Result};
use aws_config::SdkConfig;
use aws_sdk_dynamodb::{Client, config::ProvideCredentials, types::AttributeValue};
use aws_sigv4::{
    http_request::{SignableBody, SignableRequest, SigningSettings, sign},
    sign::v4,
};
use serde::Deserialize;
use std::time::{Duration, SystemTime};

const KEY: &str = "sydney-development";

#[derive(Clone)]
pub struct Store {
    client: Client,
    table: String,
}

impl Store {
    pub fn new(config: &SdkConfig, table: String) -> Self {
        Self {
            client: Client::new(config),
            table,
        }
    }

    pub async fn read(&self) -> Result<(u64, Realm)> {
        let out = self
            .client
            .get_item()
            .table_name(&self.table)
            .consistent_read(true)
            .key("realm", AttributeValue::S(KEY.into()))
            .send()
            .await?;
        let Some(item) = out.item else {
            return Ok((0, Realm::default()));
        };
        let revision = item
            .get("revision")
            .and_then(|v| v.as_n().ok())
            .ok_or("Missing revision")?
            .parse()?;
        let state = item
            .get("state")
            .and_then(|v| v.as_s().ok())
            .ok_or("Missing realm state")?;
        Ok((revision, serde_json::from_str(state)?))
    }

    pub async fn change<T>(&self, mut f: impl FnMut(&mut Realm) -> Option<T>) -> Result<Option<T>> {
        let (revision, state) = self.read().await?;
        self.change_from(revision, state, &mut f).await
    }

    pub async fn change_from<T>(
        &self,
        mut revision: u64,
        mut state: Realm,
        mut f: impl FnMut(&mut Realm) -> Option<T>,
    ) -> Result<Option<T>> {
        for attempt in 0..8 {
            if attempt != 0 {
                (revision, state) = self.read().await?;
            }
            let Some(result) = f(&mut state) else {
                return Ok(None);
            };
            let mut request = self
                .client
                .put_item()
                .table_name(&self.table)
                .item("realm", AttributeValue::S(KEY.into()))
                .item(
                    "revision",
                    AttributeValue::N(
                        revision
                            .checked_add(1)
                            .ok_or("Revision exhausted")?
                            .to_string(),
                    ),
                )
                .item("state", AttributeValue::S(serde_json::to_string(&state)?));
            request = if revision == 0 {
                request.condition_expression("attribute_not_exists(revision)")
            } else {
                request
                    .condition_expression("revision = :previous")
                    .expression_attribute_values(
                        ":previous",
                        AttributeValue::N(revision.to_string()),
                    )
            };
            match request.send().await {
                Ok(_) => return Ok(Some(result)),
                Err(error)
                    if error
                        .as_service_error()
                        .is_some_and(|e| e.is_conditional_check_failed_exception()) =>
                {
                    continue;
                }
                Err(error) => return Err(error.into()),
            }
        }
        Err("Realm state is busy".into())
    }
}

#[derive(Clone)]
pub struct Ec2 {
    config: SdkConfig,
    client: reqwest::Client,
    id: String,
}

impl Ec2 {
    pub fn new(config: SdkConfig, id: String) -> Result<Self> {
        if !id.starts_with("i-")
            || !matches!(id.len(), 10 | 19)
            || !id[2..].bytes().all(|c| c.is_ascii_hexdigit())
        {
            return Err("Invalid EC2 instance identifier".into());
        }
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .redirect(reqwest::redirect::Policy::none())
            .build()?;
        Ok(Self { config, client, id })
    }

    async fn query(&self, action: &str) -> Result<String> {
        let endpoint = "https://ec2.ap-southeast-2.amazonaws.com/";
        let body = format!(
            "Action={action}&Version=2016-11-15&InstanceId.1={}",
            self.id
        );
        let content_type = "application/x-www-form-urlencoded";
        let credentials = self
            .config
            .credentials_provider()
            .ok_or("Missing AWS credentials")?
            .provide_credentials()
            .await?;
        let identity = credentials.into();
        let params = v4::SigningParams::builder()
            .identity(&identity)
            .region("ap-southeast-2")
            .name("ec2")
            .time(SystemTime::now())
            .settings(SigningSettings::default())
            .build()?
            .into();
        let request = SignableRequest::new(
            "POST",
            endpoint,
            [("content-type", content_type)].into_iter(),
            SignableBody::Bytes(body.as_bytes()),
        )?;
        let (instructions, _) = sign(request, &params)?.into_parts();
        let mut req = self
            .client
            .post(endpoint)
            .header("content-type", content_type);
        for (name, value) in instructions.headers() {
            req = req.header(name, value);
        }
        let response = req.body(body).send().await?.error_for_status()?;
        Ok(response.text().await?)
    }

    pub async fn state(&self) -> Result<Instance> {
        let xml = self.query("DescribeInstances").await?;
        let response: Description = quick_xml::de::from_str(&xml)?;
        let mut instances = response
            .reservations
            .items
            .into_iter()
            .flat_map(|r| r.instances.items);
        let instance = instances.next().ok_or("No matching instance")?;
        if instances.next().is_some() || instance.id != self.id {
            return Err("Unexpected instance in response".into());
        }
        Ok(match instance.state.name.as_str() {
            "stopped" => Instance::Stopped,
            "pending" => Instance::Pending,
            "running" => Instance::Running,
            "stopping" => Instance::Stopping,
            _ => Instance::Other,
        })
    }

    pub async fn start(&self) -> Result<()> {
        self.query("StartInstances").await?;
        Ok(())
    }
}

#[derive(Deserialize)]
struct Description {
    #[serde(rename = "reservationSet")]
    reservations: Items<ReservationDescription>,
}
#[derive(Deserialize)]
struct Items<T> {
    #[serde(rename = "item")]
    items: Vec<T>,
}
#[derive(Deserialize)]
struct ReservationDescription {
    #[serde(rename = "instancesSet")]
    instances: Items<InstanceDescription>,
}
#[derive(Deserialize)]
struct InstanceDescription {
    #[serde(rename = "instanceId")]
    id: String,
    #[serde(rename = "instanceState")]
    state: InstanceState,
}
#[derive(Deserialize)]
struct InstanceState {
    name: String,
}

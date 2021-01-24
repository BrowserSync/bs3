use async_graphql::*;

#[derive(
    Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, async_graphql::SimpleObject,
)]
pub struct LocalUrl {
    #[serde(
        serialize_with = "crate::proxy::serialize_proxy",
        deserialize_with = "crate::proxy::deserialize_json_string"
    )]
    pub inner: url::Url,
}

impl Default for LocalUrl {
    fn default() -> Self {
        Self {
            inner: url::Url::parse("http://0.0.0.0:8080").expect("valid input"),
        }
    }
}

impl LocalUrl {
    pub fn try_from_port(port: Option<u16>) -> anyhow::Result<Self> {
        let mut local_url = Self::default();
        if let Some(port) = port {
            log::trace!("setting port {}", port);
            local_url
                .inner
                .set_port(Some(port))
                .map_err(|_e| anyhow::anyhow!("Could not set the port!"))?;
            Ok(local_url)
        } else {
            Ok(local_url)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::local_url::LocalUrl;

    #[test]
    fn test_serialize() -> anyhow::Result<()> {
        let lu = LocalUrl::try_from_port(Some(9090))?;
        let as_str = serde_json::to_string(&lu)?;
        let expected = r#"{"inner":"http://0.0.0.0:9090/"}"#;
        assert_eq!(as_str, expected);
        let as_url: LocalUrl = serde_json::from_str(expected)?;
        assert_eq!(as_url, LocalUrl::try_from_port(Some(9090))?);
        Ok(())
    }
}

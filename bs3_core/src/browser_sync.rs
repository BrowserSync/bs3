use crate::config::{default_port, get_available_port, Config};
use crate::local_url::LocalUrl;
use async_graphql::SimpleObject;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Debug, Clone, Default, Serialize, Deserialize, SimpleObject)]
pub struct BrowserSync {
    /// General configuration like which directories to serve,
    /// which proxies to setup etc
    pub config: Config,
    /// The local url/address that Browsersync will try to bind to when running the server
    /// eg: http://0.0.0.0:8080
    pub local_url: LocalUrl,
}

impl BrowserSync {
    ///
    /// Convert CLI-like arguments into valid configuration
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::path::PathBuf;
    /// # use bs3_core::browser_sync::BrowserSync;
    /// # use crate::bs3_core::serve_static::ServeStatic;
    /// # use crate::bs3_core::serve_static::{ServeStaticConfig, DirOnly};
    /// let args = vec!["fixtures/src"].into_iter();
    /// let bs = BrowserSync::try_from_args(args).expect("unpack");
    /// assert_eq!(bs.config.serve_static_config().len(), 1);
    /// assert_eq!(bs.config.serve_static_config().get(0).expect("test"), &ServeStaticConfig::DirOnly(DirOnly::from(PathBuf::from("fixtures/src"))));
    /// ```
    pub fn try_from_args(args: impl Iterator<Item = impl Into<String>>) -> anyhow::Result<Self> {
        let mut prefix = vec!["bs".to_string()];
        let args = args.into_iter().map(|m| m.into()).collect::<Vec<String>>();
        prefix.extend(args);
        let config: Config = Config::from_iter_safe(prefix)?;
        let local_url = LocalUrl::try_from_port(config.port.or_else(default_port))?;
        Ok(Self { config, local_url })
    }
    pub fn try_from_json(bs_json: impl Into<String>) -> anyhow::Result<Self> {
        let config: Config = serde_json::from_str(&bs_json.into())?;
        let local_url = LocalUrl::try_from_port(config.port.or_else(default_port))?;
        Ok(Self { config, local_url })
    }
    pub fn from_random_port() -> Self {
        let default_port = get_available_port().expect("can take a random default port");
        let mut bs = Self::default();
        bs.set_port(default_port);
        bs
    }
    pub fn set_port(&mut self, port: u16) {
        self.config.port = Some(port);
        self.local_url =
            LocalUrl::try_from_port(Some(port)).expect("Should be able to update a port");
    }
    pub fn bind_address(&self) -> String {
        let local_url = self.local_url.0.clone();
        format!(
            "{}:{}",
            local_url.host_str().expect("this part cannot can't fail"),
            local_url.port().unwrap_or(80)
        )
    }
}
#[cfg(test)]
mod test {
    use crate::browser_sync::BrowserSync;

    #[test]
    fn test_serialise() -> anyhow::Result<()> {
        let input = ". --proxy /gql~http://example.com/gql";
        let default = BrowserSync::try_from_args(input.split(' '))?;
        let as_str = serde_json::to_string_pretty(&default)?;
        let as_bs = serde_json::from_str::<BrowserSync>(as_str.as_str())?;
        assert_eq!(as_bs.config.proxy.get(0).unwrap().target.path(), "/gql");
        Ok(())
    }
}

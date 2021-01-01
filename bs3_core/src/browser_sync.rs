use crate::config::{default_port, Config};
use crate::local_url::LocalUrl;
use structopt::StructOpt;

#[derive(Debug, Clone, Default)]
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
    /// # use crate::bs3_core::serve_static::ServeStaticConfig;
    /// let args = vec!["fixtures/src"].into_iter();
    /// let bs = BrowserSync::try_from_args(args).expect("unpack");
    /// assert_eq!(bs.config.serve_static_config().len(), 1);
    /// assert_eq!(bs.config.serve_static_config().get(0).expect("test"), &ServeStaticConfig::DirOnly(PathBuf::from("fixtures/src")));
    /// ```
    pub fn try_from_args(args: impl Iterator<Item = impl Into<String>>) -> anyhow::Result<Self> {
        let mut prefix = vec!["bs".to_string()];
        let args = args.into_iter().map(|m| m.into()).collect::<Vec<String>>();
        prefix.extend(args);
        let config: Config = Config::from_iter_safe(prefix)?;
        let local_url = LocalUrl::try_from_port(config.port.or_else(default_port))?;
        Ok(Self { config, local_url })
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

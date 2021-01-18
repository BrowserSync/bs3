use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;

pub trait ServeStatic: Default {
    fn serve_static_config(&self) -> Vec<ServeStaticConfig>;
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ServeStaticConfig {
    pub routes: Option<Vec<PathBuf>>,
    pub dir: PathBuf,
}

impl ServeStaticConfig {
    pub fn new(dir: impl Into<PathBuf>, routes: Option<Vec<impl Into<PathBuf>>>) -> Self {
        let routes = routes.map(|routes| routes.into_iter().map(|r| r.into()).collect());
        Self {
            dir: dir.into(),
            routes,
        }
    }
    pub fn from_dir_only(path: impl Into<PathBuf>) -> Self {
        ServeStaticConfig {
            dir: path.into(),
            routes: None,
        }
    }
}

impl Default for ServeStaticConfig {
    fn default() -> Self {
        Self::from_dir_only(".")
    }
}

impl FromStr for ServeStaticConfig {
    type Err = ServeStaticError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split = s.split(':').collect::<Vec<&str>>();
        match split.as_slice() {
            [one] => {
                if one.is_empty() {
                    Err(ServeStaticError::Empty)
                } else {
                    Ok(ServeStaticConfig::from_dir_only(one))
                }
            }
            [rs @ .., dir] => {
                let as_routes = rs
                    .iter()
                    .map(|s| PathBuf::from(s))
                    .collect::<Vec<PathBuf>>();
                let dir = PathBuf::from(dir);
                Ok(ServeStaticConfig {
                    routes: Some(as_routes),
                    dir,
                })
            }
            _ => {
                println!("got here2");
                todo!("here")
            }
        }
    }
}

///
/// GQL types for serve static config
///
#[async_graphql::Object]
impl ServeStaticConfig {
    async fn routes(&self) -> Vec<String> {
        self.routes
            .as_ref()
            .map(|r| r.iter().map(|pb| pb.display().to_string()).collect())
            .unwrap_or_else(Vec::new)
    }
    async fn dir(&self) -> String {
        self.dir.display().to_string()
    }
}

#[derive(Error, Debug)]
pub enum ServeStaticError {
    #[error("Invalid serve static option")]
    Invalid,
    #[error("unknown serve static error")]
    Unknown,
    #[error(
        "directory path cannot be empty

    Here's an example of a valid option

    --serve-static /node_modules:node_modules

    The following is also valid where the first 3 routes will
    serve from the same directory

    --serve-static /node_modules:/nm:/nm2:node_modules
    "
    )]
    Empty,
}

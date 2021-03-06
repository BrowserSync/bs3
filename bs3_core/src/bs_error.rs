use thiserror::Error;

#[derive(Error, Debug)]
pub enum BsError {
    #[error("Could not bind to port {port} \n\n\toriginal error: {e}")]
    CouldNotBind { e: anyhow::Error, port: u16 },
    #[error("Unknown startup error \n\n\toriginal error: {e}")]
    Unknown { e: anyhow::Error },
}

impl BsError {
    pub fn unknown(
        e: impl std::error::Error + std::marker::Sync + std::marker::Send + 'static,
    ) -> anyhow::Error {
        BsError::Unknown { e: e.into() }.into()
    }
}

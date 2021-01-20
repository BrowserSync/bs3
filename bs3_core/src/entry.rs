use crate::browser_sync::BrowserSync;
use crate::start;

pub async fn from_json(json: String) -> Result<(), anyhow::Error> {
    actix_rt::System::new("bs3_core::from_json").block_on(async move {
        let bs = BrowserSync::try_from_json(json)?;
        let items = vec![bs];
        let fut = start::main(items);
        fut.await
    })
}

pub async fn from_args(
    args: impl Iterator<Item = impl Into<String>> + 'static,
) -> Result<(), anyhow::Error> {
    actix_rt::System::new("bs3_core::from_args").block_on(async move {
        let bs = BrowserSync::try_from_args(args)?;
        let items = vec![bs];
        let fut = start::main(items);
        fut.await
    })
}

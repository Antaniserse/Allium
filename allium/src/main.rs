#![feature(async_fn_in_trait)]

mod allium;
mod cores;
mod state;

use anyhow::Result;

use allium::Allium;

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let mut app = Allium::new()?;
    app.run_event_loop().await?;
    Ok(())
}
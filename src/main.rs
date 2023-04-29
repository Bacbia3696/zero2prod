use zero2prod::{
    configuration::get_configuration,
    startup::build,
    telemetry,
};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    telemetry("info");

    let configuration = get_configuration()?;
    dbg!(&configuration);

    build(configuration).await?.await?;
    Ok(())
}

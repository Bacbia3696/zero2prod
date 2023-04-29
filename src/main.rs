use zero2prod::{configuration::get_configuration, startup::Application, telemetry};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    telemetry("info");

    let configuration = get_configuration()?;
    dbg!(&configuration);

    let app = Application::build(configuration).await?;
    app.run_util_stopped().await?;
    Ok(())
}

use std::net::TcpListener;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    println!("Listen http server...");
    let listener = TcpListener::bind("127.0.0.1:8000")?;
    zero2prod::run(listener).await?.await?;
    Ok(())
}

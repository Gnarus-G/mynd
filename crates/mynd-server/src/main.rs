use std::net::SocketAddr;

use clap::Parser;
use todo::Todos;

#[derive(Parser)]
#[command(author, version, about = "Serve the Mynd PWA and API")]
struct Cli {
    #[arg(long, default_value = "127.0.0.1:4280")]
    bind: SocketAddr,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    if !cli.bind.ip().is_loopback() {
        anyhow::bail!("mynd-server only accepts a loopback bind address");
    }

    let listener = tokio::net::TcpListener::bind(cli.bind).await?;
    eprintln!("[INFO] serving Mynd at http://{}", listener.local_addr()?);
    axum::serve(listener, mynd_server::app(Todos::load_up_with_persistor()))
        .with_graceful_shutdown(shutdown())
        .await?;
    Ok(())
}

async fn shutdown() {
    let _ = tokio::signal::ctrl_c().await;
}

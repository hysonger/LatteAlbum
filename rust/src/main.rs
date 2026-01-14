use latte_album::app::App;
use latte_album::config::Config;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // 加载配置
    let config = Config::from_env()?;

    info!("Starting Latte Album server...");
    info!("Server address: {}:{}", config.host, config.port);
    info!("Photo base path: {:?}", config.base_path);

    // 创建并运行应用
    let app = App::new(config).await?;
    app.run().await?;

    Ok(())
}

use anyhow::Result;
use clap::Parser;
use readflow::ui::app::App;
use readflow::ui::runner::run_app;
use readflow::{get_cache_dir, get_config_dir, get_data_dir};
use std::str::FromStr;
use std::sync::OnceLock;
use tracing::info;
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

static LOG_GUARD: OnceLock<WorkerGuard> = OnceLock::new();

#[derive(Parser, Debug)]
#[command(name = "readflow")]
#[command(about = "A modern TUI browser - Reader's Delight", long_about = None)]
struct Args {
    #[arg(short, long, default_value = "")]
    url: String,

    #[arg(short, long, default_value = "dark")]
    theme: String,

    #[arg(short = 'k', long, default_value = "false")]
    insecure: bool,

    #[arg(short, long, default_value = "false")]
    debug: bool,
}

fn setup_logging(debug: bool) {
    let data_dir = get_data_dir();
    std::fs::create_dir_all(&data_dir).ok();

    let file_appender =
        RollingFileAppender::new(Rotation::DAILY, data_dir.join("logs"), "readflow.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    LOG_GUARD.set(guard).ok();

    let level = if debug { "debug" } else { "info" };
    let filter = tracing_subscriber::filter::LevelFilter::from_level(
        Level::from_str(level).unwrap_or(Level::INFO),
    );

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_writer(non_blocking).with_ansi(false))
        .with(fmt::layer().with_writer(std::io::stderr))
        .init();

    std::panic::set_hook(Box::new(|panic_info| {
        tracing::error!("PANIC: {}", panic_info);
    }));
}

fn main() -> Result<()> {
    let args = Args::parse();
    setup_logging(args.debug);

    info!("Starting ReadFlow v{}", env!("CARGO_PKG_VERSION"));
    info!("Data directory: {:?}", get_data_dir());
    info!("Config directory: {:?}", get_config_dir());
    info!("Cache directory: {:?}", get_cache_dir());

    std::fs::create_dir_all(get_data_dir())?;
    std::fs::create_dir_all(get_cache_dir())?;
    std::fs::create_dir_all(get_config_dir())?;

    let theme = match args.theme.to_lowercase().as_str() {
        "light" => readflow::Theme::Light,
        "dark" => readflow::Theme::Dark,
        "sepia" => readflow::Theme::Sepia,
        _ => readflow::Theme::Dark,
    };

    let mut app = App::new(theme, args.insecure);

    if !args.url.is_empty() {
        app.navigate_to(&args.url)?;
    }

    run_app(app)?;

    Ok(())
}

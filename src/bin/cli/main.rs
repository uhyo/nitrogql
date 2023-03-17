use cli::run_cli;

mod cli;
mod config_file;

fn main() -> anyhow::Result<()> {
    run_cli(std::env::args());
    Ok(())
}

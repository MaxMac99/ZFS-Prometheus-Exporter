use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about = "ZFS Prometheus Exporter", long_about = None)]
pub struct Args {
    #[arg(short, long, default_value = "9134")]
    pub port: u16,

    #[arg(short = 'H', long, default_value = "0.0.0.0")]
    pub host: String,
}

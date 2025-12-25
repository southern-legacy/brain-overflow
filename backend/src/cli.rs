use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(name = "Brain Overflow Server", bin_name = "br-ovfl")]
#[command(author = "Sylvan Lyon")]
pub struct Cli {
    /// Port of server, will override the port specified in config file
    #[arg(short = 'p', long = "port", default_value_t = 10086)]
    pub port: u16,

    /// Which configuration file to read, default to ~/.config/brain/brain-overflow.toml
    #[arg(long = "config-path")]
    pub config_path: Option<String>
}

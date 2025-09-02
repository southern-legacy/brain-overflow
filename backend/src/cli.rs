use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(name = "Brain Overflow Server", bin_name = "br-ovfl")]
#[command(author = "Sylvan Lyon")]
pub struct Cli {
    /// Port of server, will override the port specified in config file
    #[arg(short = 'p', long = "port")]
    pub port: Option<u16>,
}

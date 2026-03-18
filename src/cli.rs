use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "cugb-hpc-getkey")]
#[command(about = "CUGB HPC auto-login tool")]
#[command(version = "0.1.0")]
pub struct Cli {
    #[arg(short = 'u', long = "username", env = "HPC_USERNAME")]
    pub username: Option<String>,

    #[arg(short = 'p', long = "password", env = "HPC_PASSWORD")]
    pub password: Option<String>,

    #[arg(short = 'q', long = "quiet", default_value = "false")]
    pub quiet: bool,

    #[arg(short = 'v', long = "verbose", default_value = "false")]
    pub verbose: bool,

    #[arg(
        short = 'f',
        long = "force",
        default_value = "false",
        help = "Force re-login, ignore cache"
    )]
    pub force: bool,

    #[arg(
        short = 's',
        long = "status",
        default_value = "false",
        help = "Show cache status"
    )]
    pub status: bool,
}

impl Cli {
    pub fn get_logger_options(&self) -> crate::types::LoggerOptions {
        crate::types::LoggerOptions {
            quiet: self.quiet,
            verbose: self.verbose,
        }
    }
}

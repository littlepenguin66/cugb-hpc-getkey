use clap::Parser;

const HELP_EXAMPLES: &str = "\
examples:
  ghpc --status
  ghpc --status --json
  ghpc --force --verbose
  printf '%s\\n' \"$HPC_PASSWORD\" | ghpc --username your_user --password-stdin
  ghpc --print-token --force
";

#[derive(Parser, Debug)]
#[command(about = "CUGB HPC auto-login tool")]
#[command(version)]
#[command(after_help = HELP_EXAMPLES)]
pub struct Cli {
    #[arg(short = 'u', long = "username", env = "HPC_USERNAME", hide_env_values = true, help = "HPC username")]
    pub username: Option<String>,

    #[arg(short = 'p', long = "password", env = "HPC_PASSWORD", hide_env_values = true, help = "HPC password")]
    pub password: Option<String>,

    #[arg(short = 'q', long = "quiet", default_value = "false", conflicts_with = "verbose")]
    pub quiet: bool,

    #[arg(short = 'v', long = "verbose", default_value = "false", conflicts_with = "quiet")]
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

    #[arg(
        long = "json",
        default_value = "false",
        help = "Emit JSON output"
    )]
    pub json: bool,

    #[arg(
        long = "password-stdin",
        default_value = "false",
        help = "Read the password from stdin"
    )]
    pub password_stdin: bool,

    #[arg(
        long = "print-token",
        default_value = "false",
        help = "Print the resolved token on success"
    )]
    pub print_token: bool,
}

impl Cli {
    pub fn get_logger_options(&self) -> crate::types::LoggerOptions {
        crate::types::LoggerOptions {
            quiet: self.quiet,
            verbose: self.verbose,
            json: self.json,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Cli;
    use clap::CommandFactory;

    #[test]
    fn clap_metadata_matches_crate_metadata() {
        let command = Cli::command();

        assert_eq!(command.get_name(), env!("CARGO_PKG_NAME"));
        assert_eq!(command.get_version(), Some(env!("CARGO_PKG_VERSION")));
    }
}

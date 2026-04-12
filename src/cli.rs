use clap::Parser;

const HELP_TEMPLATE: &str = "\
{name} {version}
{about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}";

const HELP_EXAMPLES: &str = concat!(
    "examples:
  ghpc --status
  ghpc --status --json
  ghpc --force --verbose
  printf '%s\\n' \"$HPC_PASSWORD\" | ghpc --username your_user --password-stdin
  ghpc --print-token --force"
);

const LONG_VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    "\ncache-aware cugb hpc login and ssh key download cli"
);

#[derive(Parser, Debug)]
#[command(about = "cache-aware cugb hpc login and ssh key download cli")]
#[command(version)]
#[command(long_version = LONG_VERSION)]
#[command(after_help = HELP_EXAMPLES)]
#[command(help_template = HELP_TEMPLATE)]
pub struct Cli {
    #[arg(short = 'u', long = "username", env = "HPC_USERNAME", hide_env_values = true, help = "HPC username", help_heading = "input")]
    pub username: Option<String>,

    #[arg(short = 'p', long = "password", env = "HPC_PASSWORD", hide_env_values = true, help = "HPC password", help_heading = "input")]
    pub password: Option<String>,

    #[arg(short = 'q', long = "quiet", default_value = "false", conflicts_with = "verbose", help = "Hide informational success output", help_heading = "output")]
    pub quiet: bool,

    #[arg(short = 'v', long = "verbose", default_value = "false", conflicts_with = "quiet", help = "Show step logs and raw failure detail", help_heading = "output")]
    pub verbose: bool,

    #[arg(
        short = 'f',
        long = "force",
        default_value = "false",
        help = "Force re-login, ignore cache",
        help_heading = "mode"
    )]
    pub force: bool,

    #[arg(
        short = 's',
        long = "status",
        default_value = "false",
        help = "Show cache and key file status",
        help_heading = "mode"
    )]
    pub status: bool,

    #[arg(
        long = "json",
        default_value = "false",
        help = "Emit JSON output",
        help_heading = "output"
    )]
    pub json: bool,

    #[arg(
        long = "password-stdin",
        default_value = "false",
        help = "Read the password from stdin",
        help_heading = "input"
    )]
    pub password_stdin: bool,

    #[arg(
        long = "print-token",
        default_value = "false",
        help = "Print the resolved token on success",
        help_heading = "output"
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

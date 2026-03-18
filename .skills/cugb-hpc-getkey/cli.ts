import { Command } from "commander";
import { LoginConfig, LoggerOptions } from "./types";

export interface CliOptions {
  username?: string;
  password?: string;
  quiet: boolean;
  verbose: boolean;
  force: boolean;
  status: boolean;
}

export function createCommand(): Command {
  const program = new Command();

  program
    .name("hpc-login")
    .description("HPC Auto Login Tool")
    .version("1.0.0")
    .option("-u, --username <username>", "Username")
    .option("-p, --password <password>", "Password")
    .option("-q, --quiet", "Silent mode, only output token")
    .option("-v, --verbose", "Verbose logging")
    .option("-f, --force", "Force refresh, ignore cache")
    .option("-s, --status", "Show cache status");

  return program;
}

export function parseOptions(program: Command): CliOptions {
  const opts = program.opts();
  return {
    username: opts.username,
    password: opts.password,
    quiet: opts.quiet || false,
    verbose: opts.verbose || false,
    force: opts.force || false,
    status: opts.status || false,
  };
}

export function getLoginConfig(options: CliOptions): LoginConfig | null {
  const username = options.username || process.env.HPC_USERNAME;
  const password = options.password || process.env.HPC_PASSWORD;

  if (!username || !password) {
    return null;
  }

  return {
    username,
    password,
    service: "https://hpc.cugb.edu.cn/ac/api/auth/loginSsoRedirect.action",
  };
}

export function getLoggerOptions(options: CliOptions): LoggerOptions {
  return {
    quiet: options.quiet,
    verbose: options.verbose,
  };
}

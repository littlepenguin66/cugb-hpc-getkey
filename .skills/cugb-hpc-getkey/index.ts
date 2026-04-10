import {
  createCommand,
  parseOptions,
  getLoginConfig,
  getLoggerOptions,
} from "./cli";
import { readCache, writeCache, getCacheStatus } from "./cache";
import { login, downloadKey } from "./login";

type DownloadFlowResult = {
  token: string;
  source: "cache" | "login";
};

async function executeDownloadFlow(
  force: boolean,
  username: string,
  loggerOptions: ReturnType<typeof getLoggerOptions>,
  loginConfig: NonNullable<ReturnType<typeof getLoginConfig>>,
): Promise<DownloadFlowResult> {
  if (!force) {
    const cachedToken = await readCache(username);
    if (cachedToken) {
      try {
        await downloadKey(cachedToken, loggerOptions);
        return { token: cachedToken, source: "cache" };
      } catch {}
    }
  }

  const token = await login(loginConfig, loggerOptions);
  await downloadKey(token, loggerOptions);
  await writeCache(username, token);
  return { token, source: "login" };
}

function printError(prefix: string, error: unknown): void {
  console.error(prefix, error instanceof Error ? error.message : error);
}

async function main() {
  const program = createCommand();
  program.parse();

  const options = parseOptions(program);
  const loggerOptions = getLoggerOptions(options);

  if (options.status) {
    const status = await getCacheStatus();
    if (status.exists) {
      const expires = new Date(status.expiresAt!).toLocaleString();
      const validity = status.valid ? "Valid" : "Expired";
      console.log(`Cache status: ${validity}`);
      console.log(`Username: ${status.username}`);
      console.log(`Expires at: ${expires}`);
    } else {
      console.log("No cache");
    }
    return;
  }

  const config = getLoginConfig(options);
  if (!config) {
    console.error("Please set username and password:");
    console.error(
      "  Method 1: Command line arguments -u <username> -p <password>",
    );
    console.error(
      "  Method 2: Environment variables HPC_USERNAME and HPC_PASSWORD",
    );
    process.exit(1);
  }

  try {
    const result = await executeDownloadFlow(
      options.force,
      config.username,
      loggerOptions,
      config,
    );

    if (options.quiet) {
      console.log(result.token);
      return;
    }

    if (options.verbose) {
      if (result.source === "cache") {
        console.error("Using cached token");
      }
      console.log(result.token);
    }
  } catch (error) {
    printError("Operation failed:", error);
    process.exit(1);
  }
}

main().catch((error) => {
  printError("Operation failed:", error);
  process.exit(1);
});

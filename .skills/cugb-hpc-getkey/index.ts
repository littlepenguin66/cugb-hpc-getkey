import {
  createCommand,
  parseOptions,
  getLoginConfig,
  getLoggerOptions,
} from "./cli";
import { readCache, writeCache, getCacheStatus, clearCache } from "./cache";
import { login, downloadKey } from "./login";

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

  if (!options.force) {
    const cachedToken = await readCache(config.username);
    if (cachedToken) {
      if (!options.quiet) {
        console.log("Using cached token");
      }
      await downloadKey(cachedToken, loggerOptions);
      console.log(cachedToken);
      return;
    }
  }

  try {
    const token = await login(config, loggerOptions);
    await writeCache(config.username, token);
    await downloadKey(token, loggerOptions);
    console.log(token);
  } catch (error) {
    console.error(
      "Login failed:",
      error instanceof Error ? error.message : error,
    );
    process.exit(1);
  }
}

main().catch(console.error);

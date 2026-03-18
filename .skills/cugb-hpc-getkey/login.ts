import { LoginConfig, LoggerOptions } from "./types";
import { encryptPassword } from "./crypto";
import { parseCookies, getCookieString } from "./session";

const SERVICE = "https://hpc.cugb.edu.cn/ac/api/auth/loginSsoRedirect.action";

class Logger {
  constructor(private options: LoggerOptions) {}

  info(message: string): void {
    if (!this.options.quiet) {
      console.log(message);
    }
  }

  verbose(message: string): void {
    if (this.options.verbose) {
      console.log(message);
    }
  }

  error(message: string): void {
    console.error(message);
  }

  token(token: string): void {
    console.log(token);
  }
}

export async function login(
  config: LoginConfig,
  loggerOptions: LoggerOptions,
): Promise<string> {
  const logger = new Logger(loggerOptions);
  const cookies = new Map<string, string>();

  const loginPageUrl = `https://hpc.cugb.edu.cn/sso/login?service=${encodeURIComponent(SERVICE)}&t=${Date.now()}`;

  logger.verbose("Step 1: Fetching login page...");
  const loginPageRes = await fetch(loginPageUrl);
  parseCookies(loginPageRes.headers.getSetCookie(), cookies);

  const loginPageHtml = await loginPageRes.text();

  const executionMatch = loginPageHtml.match(
    /name="execution"\s+value="([^"]+)"/,
  );
  if (!executionMatch) {
    throw new Error("Failed to get execution token");
  }
  const execution = executionMatch[1];

  logger.verbose("Step 2: Encrypting password...");
  const encryptedPassword = encryptPassword(config.password);

  logger.verbose("Step 3: Sending login request...");
  const loginBody = new URLSearchParams({
    username: config.username,
    password: encryptedPassword,
    encrypted: "true",
    mode: "0",
    captcha: "",
    execution: execution,
    _eventId: "submit",
    geolocation: "",
    submit: "Login",
  });

  const loginRes = await fetch(loginPageUrl, {
    method: "POST",
    headers: {
      "Content-Type": "application/x-www-form-urlencoded",
      Cookie: getCookieString(cookies),
      Origin: "https://hpc.cugb.edu.cn",
      Referer: loginPageUrl,
    },
    body: loginBody.toString(),
    redirect: "manual",
  });

  parseCookies(loginRes.headers.getSetCookie(), cookies);

  if (loginRes.status === 302) {
    const location = loginRes.headers.get("Location");
    if (location) {
      const ticketMatch = location.match(/ticket=([^&]+)/);
      if (ticketMatch) {
        const ticket = ticketMatch[1];
        const ssoRes = await fetch(`${SERVICE}?ticket=${ticket}`, {
          headers: { Cookie: getCookieString(cookies) },
          redirect: "manual",
        });
        parseCookies(ssoRes.headers.getSetCookie(), cookies);
      }
    }
  } else if (loginRes.status === 200) {
    const loginResponseBody = await loginRes.text();
    const redirectMatch = loginResponseBody.match(
      /window\.location\.href\s*=\s*['"]([^'"]+)['"]/,
    );
    if (redirectMatch) {
      const redirectUrl = redirectMatch[1];
      const redirectRes = await fetch(redirectUrl, {
        headers: { Cookie: getCookieString(cookies) },
        redirect: "manual",
      });
      parseCookies(redirectRes.headers.getSetCookie(), cookies);

      if (redirectRes.status === 302) {
        const nextLocation = redirectRes.headers.get("Location");
        if (nextLocation) {
          const nextRes = await fetch(nextLocation, {
            headers: { Cookie: getCookieString(cookies) },
            redirect: "manual",
          });
          parseCookies(nextRes.headers.getSetCookie(), cookies);
        }
      }
    }
  } else {
    throw new Error(`Login failed, status code: ${loginRes.status}`);
  }

  logger.verbose("Step 4: Getting JWT token...");
  const tokenUrl =
    "https://hpc.cugb.edu.cn/ac/api/user/getCurrentUserInfo.action?includeToken=true&refresh=true";
  const tokenRes = await fetch(tokenUrl, {
    headers: { Cookie: getCookieString(cookies) },
  });

  const tokenText = await tokenRes.text();

  if (tokenRes.status !== 200) {
    throw new Error("Failed to get token");
  }

  const tokenData = JSON.parse(tokenText);

  if (tokenData.code !== "0" || !tokenData.data?.tokenList?.length) {
    throw new Error("Failed to get token");
  }

  return tokenData.data.tokenList[0].token;
}

export async function downloadKey(
  jwtToken: string,
  loggerOptions: LoggerOptions,
): Promise<void> {
  const logger = new Logger(loggerOptions);

  logger.verbose("Step 5: Downloading private key...");
  const apiUrl =
    "https://gridview.cugb.edu.cn:6081/sothisai/api/eshell/action/downloadkey";
  const apiRes = await fetch(apiUrl, {
    headers: {
      token: jwtToken,
      Accept: "application/json",
    },
  });

  const apiData = (await apiRes.json()) as {
    code: string;
    data?: string;
    msg?: string;
  };

  if (apiData.code === "0" && apiData.data) {
    const keyPath = `${process.env.HOME}/.hpckey`;
    await Bun.write(keyPath, apiData.data);

    const { chmodSync } = await import("fs");
    chmodSync(keyPath, 0o600);

    logger.info(`Private key saved to: ${keyPath}`);
  } else {
    logger.error(`Failed to get private key: ${apiData.msg}`);
  }
}

import type { LoginConfig, LoggerOptions } from "./types.ts";
import { encryptPassword } from "./crypto.ts";
import { parseCookies, getCookieString } from "./session.ts";
import { join } from "path";

const SERVICE = "https://hpc.cugb.edu.cn/ac/api/auth/loginSsoRedirect.action";

const HEADERS = {
  "User-Agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
  "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8",
  "Accept-Language": "zh-CN,zh;q=0.9,en;q=0.8",
};

class Logger {
  constructor(private options: LoggerOptions) {}

  info(message: string): void {
    if (!this.options.quiet) console.log(message);
  }

  verbose(message: string): void {
    if (this.options.verbose) console.log(message);
  }
}

function resolveUrl(url: string): string {
  return url.startsWith("http") ? url : `https://hpc.cugb.edu.cn${url}`;
}

function headersWithCookies(cookies: Map<string, string>): Record<string, string> {
  return { ...HEADERS, Cookie: getCookieString(cookies) };
}

async function followRedirect(
  location: string | null,
  cookies: Map<string, string>,
): Promise<void> {
  if (!location) return;
  const res = await fetch(resolveUrl(location), {
    headers: headersWithCookies(cookies),
    redirect: "manual",
  });
  parseCookies(res.headers.getSetCookie(), cookies);
}

export async function login(config: LoginConfig, loggerOptions: LoggerOptions): Promise<string> {
  const logger = new Logger(loggerOptions);
  const cookies = new Map<string, string>();

  const loginPageUrl = `https://hpc.cugb.edu.cn/sso/login?service=${encodeURIComponent(config.service)}&t=${Date.now()}`;

  logger.verbose("Step 1: Fetching login page...");
  const loginPageRes = await fetch(loginPageUrl, { headers: HEADERS });
  parseCookies(loginPageRes.headers.getSetCookie(), cookies);

  const loginPageHtml = await loginPageRes.text();
  const executionMatch = loginPageHtml.match(/name="execution"\s+value="([^"]+)"/);
  if (!executionMatch) throw new Error("Failed to get execution token");
  const execution = executionMatch[1];

  logger.verbose("Step 2: Encrypting password...");
  const encryptedPassword = await encryptPassword(config.password);

  logger.verbose("Step 3: Sending login request...");
  const loginBody = new URLSearchParams({
    username: config.username,
    password: encryptedPassword,
    encrypted: "true",
    mode: "0",
    captcha: "",
    execution,
    _eventId: "submit",
    geolocation: "",
    submit: "登录",
  });

  const loginRes = await fetch(loginPageUrl, {
    method: "POST",
    headers: {
      ...HEADERS,
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
    const ticketMatch = location?.match(/ticket=([^&]+)/);
    if (ticketMatch) {
      const ssoRes = await fetch(`${SERVICE}?ticket=${ticketMatch[1]}`, {
        headers: headersWithCookies(cookies),
        redirect: "manual",
      });
      parseCookies(ssoRes.headers.getSetCookie(), cookies);
      if (ssoRes.status === 302) {
        await followRedirect(ssoRes.headers.get("Location"), cookies);
      }
    }
  } else if (loginRes.status === 200) {
    const body = await loginRes.text();
    const redirectMatch = body.match(/window\.location\.href\s*=\s*['"]([^'"]+)['"]/);
    if (redirectMatch) {
      const redirectRes = await fetch(redirectMatch[1], {
        headers: headersWithCookies(cookies),
        redirect: "manual",
      });
      parseCookies(redirectRes.headers.getSetCookie(), cookies);
      if (redirectRes.status === 302) {
        await followRedirect(redirectRes.headers.get("Location"), cookies);
      }
    }
  } else {
    throw new Error(`Login failed, status code: ${loginRes.status}`);
  }

  logger.verbose("Step 4: Getting JWT token...");
  const tokenRes = await fetch(
    "https://hpc.cugb.edu.cn/ac/api/user/getCurrentUserInfo.action?includeToken=true&refresh=true",
    { headers: headersWithCookies(cookies) }
  );

  if (tokenRes.status !== 200) throw new Error("Failed to get token");

  const tokenData = await tokenRes.json() as { code: string; data?: { tokenList?: { token: string }[] } };
  if (tokenData.code !== "0" || !tokenData.data?.tokenList?.length) {
    throw new Error("Failed to get token");
  }

  return tokenData.data.tokenList[0].token;
}

function extractPrivateKey(data: { code: string; data?: string; msg?: string }): string {
  if (data.code === "0" && data.data) {
    return data.data;
  }

  if (data.msg) {
    throw new Error(`Failed to get private key: ${data.msg}`);
  }

  throw new Error("Failed to get private key");
}

export async function downloadKey(jwtToken: string, loggerOptions: LoggerOptions): Promise<void> {
  const logger = new Logger(loggerOptions);

  logger.verbose("Step 5: Downloading private key...");
  const res = await fetch("https://gridview.cugb.edu.cn:6081/sothisai/api/eshell/action/downloadkey", {
    headers: { token: jwtToken, Accept: "application/json" },
  });

  const data = (await res.json()) as { code: string; data?: string; msg?: string };
  const privateKey = extractPrivateKey(data);
  const home = process.env.HOME;
  if (!home) {
    throw new Error("Failed to determine home directory");
  }

  const keyPath = join(home, ".hpckey");
  await Bun.write(keyPath, privateKey);
  const { chmodSync } = await import("fs");
  chmodSync(keyPath, 0o600);
  logger.info(`Private key saved to: ${keyPath}`);
}

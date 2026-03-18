export interface LoginConfig {
  username: string;
  password: string;
  service: string;
}

export interface TokenCache {
  username: string;
  token: string;
  expiresAt: number;
  createdAt: number;
}

export interface SessionData {
  cookies: Map<string, string>;
  execution: string;
}

export interface ApiConfig {
  loginUrl: string;
  tokenUrl: string;
  downloadKeyUrl: string;
}

export interface LoggerOptions {
  quiet: boolean;
  verbose: boolean;
}
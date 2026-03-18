import { SessionData } from './types';

export function parseCookies(setCookies: string[], cookies: Map<string, string>) {
  for (const cookie of setCookies) {
    const match = cookie.match(/^([^=]+)=([^;]+)/);
    if (match) {
      cookies.set(match[1], match[2]);
    }
  }
}

export function getCookieString(cookies: Map<string, string>): string {
  return Array.from(cookies.entries())
    .map(([k, v]) => `${k}=${v}`)
    .join('; ');
}

export function createSessionData(): SessionData {
  return {
    cookies: new Map(),
    execution: '',
  };
}
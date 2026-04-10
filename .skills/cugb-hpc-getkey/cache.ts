import { TokenCache, TokenCacheFile } from './types';
import { join } from 'path';

const CACHE_FILE = join(process.env.HOME || '~', '.hpc-login-cache.json');
const DEFAULT_TTL = 2 * 60 * 60 * 1000;

function normalizeCache(cache: TokenCacheFile): TokenCache {
  const expiresAt = cache.expiresAt ?? cache.expires_at;
  const createdAt = cache.createdAt ?? cache.created_at ?? Date.now();

  if (typeof expiresAt !== 'number') {
    throw new Error('Invalid cache format');
  }

  return {
    username: cache.username,
    token: cache.token,
    expiresAt,
    createdAt,
  };
}

function serializeCache(cache: TokenCache): TokenCacheFile {
  return {
    username: cache.username,
    token: cache.token,
    expires_at: cache.expiresAt,
    created_at: cache.createdAt,
  };
}

export async function readCache(username: string): Promise<string | null> {
  try {
    const text = await Bun.file(CACHE_FILE).text();
    const cache = normalizeCache(JSON.parse(text) as TokenCacheFile);
    
    if (cache.username !== username) {
      return null;
    }
    
    if (Date.now() >= cache.expiresAt) {
      return null;
    }
    
    return cache.token;
  } catch {
    return null;
  }
}

export async function writeCache(username: string, token: string, ttlMs: number = DEFAULT_TTL): Promise<void> {
  const cache: TokenCache = {
    username,
    token,
    expiresAt: Date.now() + ttlMs,
    createdAt: Date.now(),
  };
  await Bun.write(CACHE_FILE, JSON.stringify(serializeCache(cache), null, 2));
  
  const { chmodSync } = await import('fs');
  chmodSync(CACHE_FILE, 0o600);
}

export async function clearCache(): Promise<void> {
  try {
    await Bun.file(CACHE_FILE).unlink();
  } catch {
  }
}

export async function getCacheStatus(): Promise<{ exists: boolean; valid: boolean; username?: string; expiresAt?: number }> {
  try {
    const text = await Bun.file(CACHE_FILE).text();
    const cache = normalizeCache(JSON.parse(text) as TokenCacheFile);
    const valid = Date.now() < cache.expiresAt;
    return { exists: true, valid, username: cache.username, expiresAt: cache.expiresAt };
  } catch {
    return { exists: false, valid: false };
  }
}

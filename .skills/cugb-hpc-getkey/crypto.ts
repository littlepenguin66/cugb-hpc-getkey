import crypto from 'crypto';

const LOGIN_JS_URL = 'https://hpc.cugb.edu.cn/sso/themes/sso/js/login-744fe89e6ff1efcab5fff7e1668641b0.js';
const DEFAULT_PUBLIC_KEY = 'MFwwDQYJKoZIhvcNAQEBBQADSwAwSAJBALaXEnbjI6fjy+t9W9AiO/KS0q+b/OZFS+7ykinLbiriUx9P8BcuuHnVbXNiZp5jW70eVGBtX4DhGUPzJa1YT/8CAwEAAQ==';

let cachedPublicKey: string | null = null;

function base64ToPem(base64Key: string): string {
  const lines = base64Key.match(/.{1,64}/g) || [];
  return `-----BEGIN PUBLIC KEY-----\n${lines.join('\n')}\n-----END PUBLIC KEY-----`;
}

async function fetchPublicKey(): Promise<string> {
  if (cachedPublicKey) {
    return cachedPublicKey;
  }

  try {
    const res = await fetch(LOGIN_JS_URL);
    const body = await res.text();
    const match = body.match(/var key = '([^']+)'/);
    if (!match) {
      throw new Error('Failed to extract public key from login.js');
    }

    cachedPublicKey = match[1];
    return match[1];
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    console.error(`Warning: Failed to fetch public key: ${message}. Using fallback public key`);
  }

  return DEFAULT_PUBLIC_KEY;
}

function encryptPasswordWithPublicKey(password: string, publicKeyBase64: string): string {
  const publicKey = base64ToPem(publicKeyBase64);
  const buffer = Buffer.from(password, 'utf-8');

  try {
    const encrypted = crypto.publicEncrypt(
      {
        key: publicKey,
        padding: crypto.constants.RSA_PKCS1_PADDING,
      },
      buffer
    );

    return encrypted.toString('base64');
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    throw new Error(`Failed to encrypt password: ${message}`);
  }
}

export async function encryptPassword(password: string): Promise<string> {
  const publicKeyBase64 = await fetchPublicKey();
  return encryptPasswordWithPublicKey(password, publicKeyBase64);
}

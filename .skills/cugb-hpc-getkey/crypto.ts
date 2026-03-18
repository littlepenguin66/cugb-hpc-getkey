import crypto from 'crypto';

const RSA_PUBLIC_KEY = 'MFwwDQYJKoZIhvcNAQEBBQADSwAwSAJBALaXEnbjI6fjy+t9W9AiO/KS0q+b/OZFS+7ykinLbiriUx9P8BcuuHnVbXNiZp5jW70eVGBtX4DhGUPzJa1YT/8CAwEAAQ==';

function base64ToPem(base64Key: string): string {
  const lines = base64Key.match(/.{1,64}/g) || [];
  return `-----BEGIN PUBLIC KEY-----\n${lines.join('\n')}\n-----END PUBLIC KEY-----`;
}

export function encryptPassword(password: string): string {
  const publicKey = base64ToPem(RSA_PUBLIC_KEY);
  const buffer = Buffer.from(password, 'utf-8');
  const encrypted = crypto.publicEncrypt(
    {
      key: publicKey,
      padding: crypto.constants.RSA_PKCS1_PADDING,
    },
    buffer
  );
  return encrypted.toString('base64');
}
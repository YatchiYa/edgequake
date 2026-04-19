import packageInfo from '../../package.json';

/**
 * Returns the current app version with a consistent leading "v".
 * Uses a public build-time override when provided, otherwise falls back
 * to the package version so release labels stay in sync automatically.
 */
export function getAppVersion(explicitVersion?: string): string {
  const rawVersion = explicitVersion ?? process.env.NEXT_PUBLIC_APP_VERSION ?? packageInfo.version ?? '0.1.0';
  const normalized = rawVersion.trim().replace(/^v/i, '');
  return `v${normalized}`;
}

export const APP_VERSION = getAppVersion();

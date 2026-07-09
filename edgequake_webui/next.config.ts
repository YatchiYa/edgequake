import type { NextConfig } from "next";
import { DEFAULT_MAX_UPLOAD_BYTES } from "./src/lib/api/upload-limits";
import { resolveDevProxyBackend } from "./src/lib/server/dev-proxy-backend";

/** Match upload-timeout.ts MAX_TIMEOUT_MS — dev rewrites proxy large PDF admits. */
const DEV_PROXY_TIMEOUT_MS = 600_000;

/** Next.js dev proxy default is 10MB; EdgeQuake allows 50MB uploads (SPEC-038). */
const DEV_PROXY_MAX_BODY = `${Math.ceil(DEFAULT_MAX_UPLOAD_BYTES / (1024 * 1024))}mb`;

const nextConfig: NextConfig = {
  // ============================================================================
  // Build Performance Optimization
  // Prevents CPU overload during compilation
  // ============================================================================

  // Limit experimental workers to prevent CPU overload
  experimental: {
    // Reduce worker count to prevent memory/CPU exhaustion
    cpus: Math.min(4, typeof process !== "undefined" && process.env.CI ? 2 : 4),
    // Use SWC minifier (faster than Terser)
    webpackBuildWorker: true,
    // SPEC-038: default rewrite proxy is 30s / 10MB — large PDFs hang at "Saving to workspace…"
    proxyTimeout: DEV_PROXY_TIMEOUT_MS,
    proxyClientMaxBodySize: DEV_PROXY_MAX_BODY,
  },

  // TypeScript configuration
  typescript: {
    // Don't fail build on TS errors (we use tsc separately)
    ignoreBuildErrors: false,
  },

  // Enable Turbopack for faster builds (Next.js 16+)
  // Turbopack is enabled by default with `next build` in Next.js 16

  // Output configuration
  output: "standalone",

  // Dev proxy: utoipa serves /swagger-ui/ (with slash); Next default strips trailing
  // slashes (308) → infinite redirect loop with backend (303). Disable for proxied paths.
  skipTrailingSlashRedirect: true,

  // Reduce logging
  logging: {
    fetches: {
      fullUrl: false,
    },
  },

  // Dev proxy: browser uses relative /api/v1 (same origin as :3001 UI).
  // Avoids NEXT_PUBLIC_API_URL port drift when backend auto-selects :8081.
  async rewrites() {
    if (process.env.NODE_ENV !== "development") {
      return [];
    }
    const backend = resolveDevProxyBackend();
    return [
      { source: "/api/:path*", destination: `${backend}/api/:path*` },
      { source: "/api-docs/:path*", destination: `${backend}/api-docs/:path*` },
      { source: "/health", destination: `${backend}/health` },
      { source: "/ready", destination: `${backend}/ready` },
      { source: "/live", destination: `${backend}/live` },
      { source: "/ws/:path*", destination: `${backend}/ws/:path*` },
    ];
  },
};

export default nextConfig;

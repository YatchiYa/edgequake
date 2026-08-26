import type { NextConfig } from "next";
import { DEFAULT_MAX_UPLOAD_BYTES } from "./src/lib/api/upload-limits";
import { resolveDevProxyBackend } from "./src/lib/server/dev-proxy-backend";

/** Match upload-timeout.ts MAX_TIMEOUT_MS — dev rewrites proxy large PDF admits. */
const DEV_PROXY_TIMEOUT_MS = 600_000;

const nextConfig: NextConfig = {
  // ============================================================================
  // Build Performance Optimization
  // Prevents CPU overload during compilation
  // ============================================================================

  // ============================================================================
  // SSE STREAMING FIX
  // ============================================================================
  // Next.js gzips proxied responses by default (`compress: true`). The gzip
  // encoder buffers the whole body, so `text/event-stream` responses arrive as
  // ONE chunk at the end instead of token-by-token — the API itself does NOT
  // compress SSE (verified: no content-encoding on :8090, `Content-Encoding:
  // gzip` added by Next on :3010). Disabling Next compression restores real
  // streaming in the browser. In production, terminate compression at the
  // reverse proxy AND exclude text/event-stream there.
  compress: false,

  // Limit experimental workers to prevent CPU overload
  experimental: {
    // Reduce worker count to prevent memory/CPU exhaustion
    cpus: Math.min(4, typeof process !== "undefined" && process.env.CI ? 2 : 4),
    // Use SWC minifier (faster than Terser)
    webpackBuildWorker: true,
    // SPEC-038: default rewrite proxy is 30s / 10MB — large PDFs hang at "Saving to workspace…"
    // Use numeric bytes (SizeLimit). Template strings like `${n}mb` widen to `string`
    // and fail `next build` typecheck (release-docker CD flake on Next 16.2).
    proxyTimeout: DEV_PROXY_TIMEOUT_MS,
    proxyClientMaxBodySize: DEFAULT_MAX_UPLOAD_BYTES,
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

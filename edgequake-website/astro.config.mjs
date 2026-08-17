import react from "@astrojs/react";
import sitemap from "@astrojs/sitemap";
import starlight from "@astrojs/starlight";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "astro/config";

export default defineConfig({
  site: "https://edgequake.com",
  trailingSlash: "always",
  output: "static",

  integrations: [
    starlight({
      title: "EdgeQuake",
      description:
        "Graph-RAG framework — Built to Ship. Knowledge graph engine powered by Rust.",
      head: [
        // Default OG image for all docs pages (PNG required for Twitter/LinkedIn/Facebook)
        {
          tag: "meta",
          attrs: { property: "og:image", content: "https://edgequake.com/og-default.png" },
        },
        {
          tag: "meta",
          attrs: { property: "og:image:width", content: "1200" },
        },
        {
          tag: "meta",
          attrs: { property: "og:image:height", content: "630" },
        },
        {
          tag: "meta",
          attrs: { property: "og:image:type", content: "image/png" },
        },
        {
          tag: "meta",
          attrs: { name: "twitter:card", content: "summary_large_image" },
        },
        {
          tag: "meta",
          attrs: { name: "twitter:site", content: "@raphaelmansuy" },
        },
        {
          tag: "meta",
          attrs: { name: "twitter:creator", content: "@raphaelmansuy" },
        },
        {
          tag: "meta",
          attrs: { name: "twitter:image", content: "https://edgequake.com/og-default.png" },
        },
      ],
      social: [
        {
          icon: "github",
          label: "GitHub",
          href: "https://github.com/raphaelmansuy/edgequake",
        },
      ],
      editLink: {
        baseUrl:
          "https://github.com/raphaelmansuy/edgequake/edit/edgequake-main/",
      },
      components: {
        Header: "./src/components/overrides/StarlightHeader.astro",
      },
      routeMiddleware: "./src/route-data.ts",
      customCss: ["./src/styles/global.css"],
      lastUpdated: true,
      pagination: true,
      disable404Route: true,
      sidebar: [
        {
          label: "Getting Started",
<<<<<<< HEAD
          autogenerate: { directory: "docs/getting-started" },
        },
        {
          label: "Concepts",
          autogenerate: { directory: "docs/concepts" },
        },
        {
          label: "Architecture",
          autogenerate: { directory: "docs/architecture" },
        },
        {
          label: "Tutorials",
          autogenerate: { directory: "docs/tutorials" },
        },
        {
          label: "API Reference",
          autogenerate: { directory: "docs/api-reference" },
=======
          items: [{ autogenerate: { directory: "docs/getting-started" } }],
        },
        {
          label: "Concepts",
          items: [{ autogenerate: { directory: "docs/concepts" } }],
        },
        {
          label: "Architecture",
          items: [{ autogenerate: { directory: "docs/architecture" } }],
        },
        {
          label: "Tutorials",
          items: [{ autogenerate: { directory: "docs/tutorials" } }],
        },
        {
          label: "API Reference",
          items: [{ autogenerate: { directory: "docs/api-reference" } }],
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
        },
        {
          label: "Deep Dives",
          collapsed: true,
<<<<<<< HEAD
          autogenerate: { directory: "docs/deep-dives" },
=======
          items: [{ autogenerate: { directory: "docs/deep-dives" } }],
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
        },
        {
          label: "Operations",
          collapsed: true,
<<<<<<< HEAD
          autogenerate: { directory: "docs/operations" },
        },
        {
          label: "Integrations",
          autogenerate: { directory: "docs/integrations" },
=======
          items: [{ autogenerate: { directory: "docs/operations" } }],
        },
        {
          label: "Integrations",
          items: [{ autogenerate: { directory: "docs/integrations" } }],
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
        },
        {
          label: "Comparisons",
          collapsed: true,
<<<<<<< HEAD
          autogenerate: { directory: "docs/comparisons" },
        },
        {
          label: "Security",
          autogenerate: { directory: "docs/security" },
        },
        {
          label: "Troubleshooting",
          autogenerate: { directory: "docs/troubleshooting" },
=======
          items: [{ autogenerate: { directory: "docs/comparisons" } }],
        },
        {
          label: "Security",
          items: [{ autogenerate: { directory: "docs/security" } }],
        },
        {
          label: "Troubleshooting",
          items: [{ autogenerate: { directory: "docs/troubleshooting" } }],
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
        },
        {
          label: "Resources",
          items: [
            { slug: "docs/cookbook" },
            { slug: "docs/faq" },
            { slug: "docs/features" },
          ],
        },
      ],
    }),
    react(),
    sitemap(),
  ],

  vite: {
    plugins: [tailwindcss()],
  },
});

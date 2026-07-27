import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";
import { SvelteKitPWA } from "@vite-pwa/sveltekit";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    sveltekit(),
    SvelteKitPWA({
      registerType: "prompt",
      includeAssets: ["favicon.svg", "icons/*.png", "icons/logo.svg"],
      manifest: {
        name: "Mynd",
        short_name: "Mynd",
        description: "A fast, private todo capture tool.",
        theme_color: "#17221c",
        background_color: "#17221c",
        display: "standalone",
        scope: "/",
        start_url: "/",
        icons: [
          { src: "/icons/icon-192.png", sizes: "192x192", type: "image/png" },
          { src: "/icons/icon-512.png", sizes: "512x512", type: "image/png" },
          {
            src: "/icons/icon-maskable-512.png",
            sizes: "512x512",
            type: "image/png",
            purpose: "maskable",
          },
        ],
      },
      workbox: {
        navigateFallback: "/",
        runtimeCaching: [
          {
            urlPattern: /\/api\//,
            handler: "NetworkOnly",
          },
        ],
      },
    }),
  ],
  server: {
    port: 5173,
    strictPort: true,
    proxy: {
      "/api": "http://127.0.0.1:4280",
    },
  },
});

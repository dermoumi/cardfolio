import { storybookTest } from "@storybook/addon-vitest/vitest-plugin";
import { playwright } from "@vitest/browser-playwright";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vitest/config";

const dirname = path.dirname(fileURLToPath(import.meta.url));

export default defineConfig({
  optimizeDeps: {
    include: ["react/jsx-dev-runtime"],
  },
  test: {
    globals: true,
    projects: [{
      extends: true,
      plugins: [
        storybookTest({
          configDir: path.join(dirname, ".storybook"),
        }),
      ],
      test: {
        name: "storybook",
        browser: {
          enabled: true,
          headless: true,
          provider: playwright({}),
          instances: [{ browser: "chromium" }],
        },
        setupFiles: [".storybook/vitest.setup.ts"],
      },
    }, {
      extends: true,
      test: {
        name: "unit",
        environment: "jsdom",
      },
    }],
  },
  css: {
    modules: {
      localsConvention: "camelCase",
    },
  },
});

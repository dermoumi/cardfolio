import type { Preview } from "@storybook/react-vite";

import UiProvider from "../src/providers/UiProvider";
import "../src/main.css";

const getPreferredColorScheme = (): "light" | "dark" => {
  if (typeof window === "undefined") return "light";
  return window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
};

const preview: Preview = {
  tags: ["autodocs"],
  parameters: {
    controls: {
      matchers: {
        color: /(background|color)$/i,
        date: /Date$/i,
      },
      disableSaveFromUI: true,
    },
    a11y: {
      test: "error",
    },
  },
  initialGlobals: {
    backgrounds: { value: getPreferredColorScheme() },
  },
  decorators: [
    (Story, { globals }) => (
      <UiProvider updateRootDataset colorScheme={globals.backgrounds?.value}>
        <Story />
      </UiProvider>
    ),
  ],
};

export default preview;

import type { Preview } from "@storybook/react-vite";

import UiProvider from "../src/providers/UiProvider";
import "../src/main.css";

const getPreferredColorScheme = (): "light" | "dark" => {
  if (typeof window === "undefined") return "light";
  return window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
};

const getPreferredMotionPreference = (): "off" | "full" | "reduced" => {
  if (typeof window === "undefined") return "full";
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches
    ? "reduced"
    : "full";
};

const preview: Preview = {
  tags: ["autodocs"],
  parameters: {
    layout: "centered",
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
  globalTypes: {
    motionPreference: {
      description: "Motion preference for components",
      toolbar: {
        title: "Motion Preference",
        icon: "play",
        dynamicTitle: true,
        items: [
          { value: "off", title: "Disable motion", icon: "stopalt" },
          { value: "full", title: "Full motion", icon: "fastforward" },
          { value: "reduced", title: "Reduced motion", icon: "playnext" },
        ],
      },
    },
  },
  initialGlobals: {
    motionPreference: getPreferredMotionPreference(),
    backgrounds: { value: getPreferredColorScheme() },
  },
  decorators: [
    (Story, { globals }) => (
      <UiProvider
        updateRootDataset
        colorScheme={globals.backgrounds?.value}
        motionPreference={globals.motionPreference}
      >
        <Story />
      </UiProvider>
    ),
  ],
};

export default preview;

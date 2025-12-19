import type { Preview } from "@storybook/react-vite";

import UiProvider from "../src/providers/UiProvider";

import "../src/main.css";
import "./preview.css";

const preview: Preview = {
  tags: ["autodocs"],
  parameters: {
    controls: {
      matchers: {
        color: /(background|color)$/i,
        date: /Date$/i,
      },
    },
    a11y: {
      test: "error",
    },
  },
  decorators: [
    (Story) => (
      <UiProvider updateRootDataset>
        <Story />
      </UiProvider>
    ),
  ],
};

export default preview;

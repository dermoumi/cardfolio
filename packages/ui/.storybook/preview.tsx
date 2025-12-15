import type { Preview } from "@storybook/react-vite";

import UiProvider from "../src/providers/UiProvider";
import "./preview.css";

const preview: Preview = {
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

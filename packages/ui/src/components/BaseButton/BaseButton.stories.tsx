import type { Meta, StoryObj } from "@storybook/react-vite";

import BaseButton from ".";

const meta = {
  component: BaseButton,
} satisfies Meta<typeof BaseButton>;
export default meta;

type Story = StoryObj<typeof meta>;

export const NormalButton: Story = {
  args: {
    children: "Click me",
  },
};

import type { Meta, StoryObj } from "@storybook/react-vite";

import { fn } from "storybook/test";

import BaseButton from ".";

const meta = {
  title: "Atoms/BaseButton",
  component: BaseButton,
  parameters: { layout: "centered" },
  args: {
    onClick: fn(),
  },
  argTypes: {
    disabled: { control: "boolean" },
    children: { description: "Content to be displayed inside the button." },
  },
} satisfies Meta<typeof BaseButton>;
export default meta;

type Story = StoryObj<typeof meta>;

export const NormalButton: Story = {
  args: {
    children: "Click me",
  },
};

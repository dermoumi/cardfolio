import type { Meta, StoryObj } from "@storybook/react-vite";

import { expect, fn } from "storybook/test";

import BaseButton from ".";

const meta = {
  title: "Atoms/BaseButton",
  component: BaseButton,
  parameters: { layout: "centered" },
  args: {
    onClick: fn(),
    children: "Button",
  },
  argTypes: {
    disabled: { control: "boolean" },
    children: { description: "Content to be displayed inside the button." },
  },
} satisfies Meta<typeof BaseButton>;
export default meta;

type Story = StoryObj<typeof meta>;

export const ClickableButton: Story = {
  args: {
    children: "Click me",
  },
  play: async ({ args, canvas, userEvent }) => {
    const button = canvas.getByText(args.children as string);
    await userEvent.click(button);

    await expect(args.onClick).toHaveBeenCalled();
  },
};

export const DisabledButton: Story = {
  args: {
    children: "You can't click me",
    disabled: true,
  },
  play: async ({ args, canvas, userEvent }) => {
    const button = canvas.getByText(args.children as string);
    await userEvent.click(button);

    await expect(args.onClick).not.toHaveBeenCalled();
  },
};

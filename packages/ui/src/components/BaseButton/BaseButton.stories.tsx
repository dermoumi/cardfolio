import type { Meta, StoryObj } from "@storybook/react-vite";

import { expect, fn } from "storybook/test";

import BaseButton from "./BaseButton";
import { RADIUS_CLASSES, SIZE_CLASSES, VARIANT_CLASSES } from "./variants";

const meta = {
  title: "Atoms/BaseButton",
  component: BaseButton,
  args: {
    onClick: fn(),
    children: "Button",
    variant: "primary",
    size: "md",
    radius: "full",
    disabled: false,
  },
  argTypes: {
    disabled: { control: "boolean" },
    variant: { control: "radio", options: Object.keys(VARIANT_CLASSES) },
    size: { control: "radio", options: Object.keys(SIZE_CLASSES) },
    radius: { control: "radio", options: Object.keys(RADIUS_CLASSES) },
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
    const button = canvas.getByRole("button");
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
    const button = canvas.getByRole("button");
    await userEvent.click(button);

    await expect(args.onClick).not.toHaveBeenCalled();
  },
};

import type { Meta, StoryObj } from "@storybook/react-vite";
import type { FC } from "react";
import type { FlexProps } from "./Flex";

import FlexComponent from "./Flex";
import { GAP_CLASSES } from "./variants";

const StoryComponent: FC<FlexProps> = (props) => {
  return (
    <FlexComponent {...props}>
      <div style={{ backgroundColor: "#E0E0E0", padding: "8px" }}>Item 1</div>
      <div style={{ backgroundColor: "#C0C0C0", padding: "8px" }}>Item 2</div>
      <div style={{ backgroundColor: "#A0A0A0", padding: "8px" }}>Item 3</div>
    </FlexComponent>
  );
};

const meta = {
  title: "Atoms/Flex",
  component: StoryComponent,
  args: {
    gap: "md",
    vertical: false,
  },
  argTypes: {
    gap: { control: "radio", options: Object.keys(GAP_CLASSES) },
  },
} satisfies Meta<typeof FlexComponent>;
export default meta;

type Story = StoryObj<typeof meta>;

export const FlexHorizontal: Story = {
  args: {
    vertical: false,
  },
};

export const FlexVertical: Story = {
  args: {
    vertical: true,
  },
};

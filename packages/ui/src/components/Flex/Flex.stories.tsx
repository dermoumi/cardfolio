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
    stretch: false,
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

const GrowStoryComponent: FC<FlexProps> = ({ vertical, ...props }) => {
  const style = {
    border: "1px solid #CCC",
    height: vertical ? "200px" : undefined,
    width: vertical ? undefined : "400px",
  };

  return (
    <div style={style}>
      <FlexComponent vertical={vertical} {...props}>
        <FlexComponent.Grow>
          <div style={{ backgroundColor: "#E0E0E0", padding: "8px" }}>Item 1</div>
        </FlexComponent.Grow>
        <div style={{ backgroundColor: "#C0C0C0", padding: "8px" }}>Item 2</div>
        <div style={{ backgroundColor: "#A0A0A0", padding: "8px" }}>Item 3</div>
      </FlexComponent>
    </div>
  );
};

export const FlexWithGrow: Story = {
  render: (args) => <GrowStoryComponent {...args} />,
  args: {
    stretch: true,
  },
};

const ShrinkStoryComponent: FC<FlexProps> = ({ vertical, ...props }) => {
  const style = {
    border: "1px solid #CCC",
    height: vertical ? "200px" : undefined,
    width: vertical ? undefined : "400px",
  };

  return (
    <div style={style}>
      <FlexComponent vertical={vertical} {...props}>
        <FlexComponent.Shrink>
          <div style={{ backgroundColor: "#E0E0E0", padding: "8px" }}>Item 1</div>
        </FlexComponent.Shrink>
        <div style={{ backgroundColor: "#C0C0C0", padding: "8px" }}>Item 2</div>
        <div style={{ backgroundColor: "#A0A0A0", padding: "8px" }}>Item 3</div>
      </FlexComponent>
    </div>
  );
};

export const FlexWithShrink: Story = {
  render: (args) => <ShrinkStoryComponent {...args} />,
  args: {
    stretch: true,
  },
};

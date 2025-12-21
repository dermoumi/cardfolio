import type { Meta, StoryObj } from "@storybook/react-vite";
import type { FC } from "react";
import type { FlexProps } from "./Flex";

import Flex from "./Flex";
import FlexGrow from "./FlexGrow";
import FlexShrink from "./FlexShrink";
import { GAP_CLASSES } from "./variants";

const StoryComponent: FC<FlexProps> = (props) => {
  return (
    <Flex {...props}>
      <div style={{ backgroundColor: "#E0E0E0", padding: "8px" }}>Item 1</div>
      <div style={{ backgroundColor: "#C0C0C0", padding: "8px" }}>Item 2</div>
      <div style={{ backgroundColor: "#A0A0A0", padding: "8px" }}>Item 3</div>
    </Flex>
  );
};

const meta = {
  title: "Atoms/Flex",
  component: Flex,
  render: (args) => <StoryComponent {...args} />,
  args: {
    gap: "md",
    stretch: false,
    vertical: false,
  },
  argTypes: {
    gap: { control: "radio", options: Object.keys(GAP_CLASSES) },
    className: { control: "text" },
  },
} satisfies Meta<typeof Flex>;
export default meta;

type Story = StoryObj<typeof meta>;

export const Horizontal: Story = {
  args: {
    vertical: false,
  },
};

export const Vertical: Story = {
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
      <Flex vertical={vertical} {...props}>
        <FlexGrow>
          <div style={{ backgroundColor: "#E0E0E0", padding: "8px" }}>Item 1</div>
        </FlexGrow>
        <div style={{ backgroundColor: "#C0C0C0", padding: "8px" }}>Item 2</div>
        <div style={{ backgroundColor: "#A0A0A0", padding: "8px" }}>Item 3</div>
      </Flex>
    </div>
  );
};

export const WithGrow: Story = {
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
      <Flex vertical={vertical} {...props}>
        <FlexShrink>
          <div style={{ backgroundColor: "#E0E0E0", padding: "8px" }}>Item 1</div>
        </FlexShrink>
        <div style={{ backgroundColor: "#C0C0C0", padding: "8px" }}>Item 2</div>
        <div style={{ backgroundColor: "#A0A0A0", padding: "8px" }}>Item 3</div>
      </Flex>
    </div>
  );
};

export const WithShrink: Story = {
  render: (args) => <ShrinkStoryComponent {...args} />,
  args: {
    stretch: true,
  },
};

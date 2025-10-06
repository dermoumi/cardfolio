import { render } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import Button from ".";
import styles from "./Button.module.css";

describe("Button", () => {
  it("matches snapshot", () => {
    const { asFragment } = render(<Button>Button</Button>);
    expect(asFragment()).toMatchSnapshot();
  });

  it("matches snapshot with icon", () => {
    const { asFragment } = render(<Button icon="network">Button</Button>);
    expect(asFragment()).toMatchSnapshot();
  });

  it("has .icon-only class when no children", () => {
    const { getByRole } = render(<Button icon="network" label="Button" />);
    expect(getByRole("button").classList).toContain(styles.iconOnly);
  });

  it("calls onClick when clicked", () => {
    const onClick = vi.fn();
    const { getByText } = render(<Button onClick={onClick}>Button</Button>);

    getByText("Button").click();
    expect(onClick).toHaveBeenCalled();
  });
});

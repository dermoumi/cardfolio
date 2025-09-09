import { render } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import Button from ".";

describe("Button", () => {
  it("matches snapshot", () => {
    const { asFragment } = render(<Button>Button</Button>);
    expect(asFragment()).toMatchSnapshot();
  });

  it("calls onClick when clicked", () => {
    const onClick = vi.fn();
    const { getByText } = render(<Button onClick={onClick}>Button</Button>);

    getByText("Button").click();
    expect(onClick).toHaveBeenCalled();
  });
});

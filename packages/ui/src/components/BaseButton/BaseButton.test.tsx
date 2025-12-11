import { render } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import BaseButton from ".";

describe("BaseButton", () => {
  it("matches snapshot", () => {
    const { asFragment } = render(<BaseButton>Button</BaseButton>);
    expect(asFragment()).toMatchSnapshot();
  });

  it("calls onClick when clicked", () => {
    const onClick = vi.fn();
    const { getByText } = render(<BaseButton onClick={onClick}>Button</BaseButton>);

    getByText("Button").click();
    expect(onClick).toHaveBeenCalled();
  });

  it("does not call onClick when disabled", () => {
    const onClick = vi.fn();
    const { getByText } = render(<BaseButton onClick={onClick} disabled>Button</BaseButton>);

    getByText("Button").click();
    expect(onClick).not.toHaveBeenCalled();
  });
});

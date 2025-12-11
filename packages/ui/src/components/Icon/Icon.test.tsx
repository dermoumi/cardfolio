import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import Icon from "./Icon";

describe("Icon component", () => {
  it("should render correctly", () => {
    const { container } = render(<Icon name="network" />);
    expect(container).toMatchSnapshot();
  });

  it("should add aria-label when provided", () => {
    const { container } = render(<Icon name="network" label="Network Icon" />);

    const iconElement = container.querySelector("svg");
    expect(iconElement?.ariaLabel).toBe("Network Icon");
  });
});

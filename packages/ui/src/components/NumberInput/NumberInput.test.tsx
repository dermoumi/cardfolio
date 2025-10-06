import { fireEvent, render } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import NumberInput from ".";

describe("NumberInput", () => {
  it("matches snapshot", () => {
    const { asFragment } = render(<NumberInput name="test" placeholder="Enter text" />);
    expect(asFragment()).toMatchSnapshot();
  });

  it("calls onChange when text is entered", () => {
    const handleChange = vi.fn();
    const { getByPlaceholderText } = render(
      <NumberInput name="test" placeholder="Enter text" onChange={handleChange} />,
    );

    const input = getByPlaceholderText("Enter text") as HTMLInputElement;
    fireEvent.change(input, { target: { value: "144" } });

    expect(handleChange).toHaveBeenCalledWith(144);
  });
});

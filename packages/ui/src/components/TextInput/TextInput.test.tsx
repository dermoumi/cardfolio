import { fireEvent, render } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import TextInput from ".";

describe("TextInput", () => {
  it("calls onChange when text is entered", () => {
    const handleChange = vi.fn();
    const { getByPlaceholderText } = render(
      <TextInput placeholder="Enter text" onChange={handleChange} />,
    );

    const input = getByPlaceholderText("Enter text") as HTMLInputElement;
    fireEvent.change(input, { target: { value: "Hello" } });

    expect(handleChange).toHaveBeenCalled();
    expect(input.value).toBe("Hello");
  });
});

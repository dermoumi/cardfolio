import type { FC } from "react";
import type { ColorSchemeContextType } from "./context";

import { render } from "@testing-library/react";
import { act, useEffect } from "react";
import { afterEach, beforeAll, describe, expect, it, vi } from "vitest";

import { MediaQueryMock } from "../../test-utils/mediaQueryMock";
import ColorSchemeProvider from "./ColorSchemeProvider";
import { useColorScheme } from "./context";

// Helper component to expose context values for testing
type TestComponentProps = {
  onMount?: (values: ColorSchemeContextType) => void;
};

const TestComponent: FC<TestComponentProps> = ({ onMount }) => {
  const values = useColorScheme();

  useEffect(() => {
    onMount?.(values);
  }, [values, onMount]);

  return <div data-testid="color-scheme-display">{values.colorScheme}</div>;
};

describe("ColorSchemeProvider", () => {
  beforeAll(() => {
    MediaQueryMock.installMock();
  });

  afterEach(() => {
    MediaQueryMock.clear();
    vi.clearAllMocks();

    // Clear root dataset before each test
    delete document.documentElement.dataset.colorScheme;
  });

  it("correctly detects system color scheme preferences", () => {
    const { getByTestId } = render(
      <ColorSchemeProvider>
        <TestComponent />
      </ColorSchemeProvider>,
    );

    // Default to light
    expect(getByTestId("color-scheme-display").textContent).toBe("light");

    // Change system preference to dark
    act(() => MediaQueryMock.setMatchingQueries("(prefers-color-scheme: dark)"));
    expect(getByTestId("color-scheme-display").textContent).toBe("dark");

    // Change system preference back to light
    act(() => MediaQueryMock.setMatchingQueries("(prefers-color-scheme: light)"));
    expect(getByTestId("color-scheme-display").textContent).toBe("light");
  });

  it("overrides system preferences when colorScheme prop is set", () => {
    MediaQueryMock.setMatchingQueries("(prefers-color-scheme: dark)");
    const { getByTestId } = render(
      <ColorSchemeProvider colorScheme="light">
        <TestComponent />
      </ColorSchemeProvider>,
    );

    // Should use light mode despite system preference for dark
    expect(getByTestId("color-scheme-display").textContent).toBe("light");
  });

  it("overrides system preferences when forced color scheme is set", () => {
    MediaQueryMock.setMatchingQueries("(prefers-color-scheme: dark)");
    let colorSchemeValue: ColorSchemeContextType;

    const { getByTestId } = render(
      <ColorSchemeProvider>
        <TestComponent
          onMount={(values) => {
            colorSchemeValue = values;
          }}
        />
      </ColorSchemeProvider>,
    );

    expect(getByTestId("color-scheme-display").textContent).toBe("dark");

    // Force light mode
    act(() => colorSchemeValue.setForcedColorScheme("light"));
    expect(getByTestId("color-scheme-display").textContent).toBe("light");
  });

  it("prefers forced color scheme over colorScheme prop", () => {
    MediaQueryMock.setMatchingQueries("(prefers-color-scheme: dark)");
    let colorSchemeValue: ColorSchemeContextType;

    const { getByTestId } = render(
      <ColorSchemeProvider colorScheme="dark">
        <TestComponent
          onMount={(values) => {
            colorSchemeValue = values;
          }}
        />
      </ColorSchemeProvider>,
    );

    expect(getByTestId("color-scheme-display").textContent).toBe("dark");

    // Force light mode
    act(() => colorSchemeValue.setForcedColorScheme("light"));
    expect(getByTestId("color-scheme-display").textContent).toBe("light");
  });

  it("reverts to system preferences when forced color scheme set to null", () => {
    MediaQueryMock.setMatchingQueries("(prefers-color-scheme: dark)");
    let colorSchemeValue: ColorSchemeContextType;

    const { getByTestId } = render(
      <ColorSchemeProvider>
        <TestComponent
          onMount={(values) => {
            colorSchemeValue = values;
          }}
        />
      </ColorSchemeProvider>,
    );

    expect(getByTestId("color-scheme-display").textContent).toBe("dark");

    // Force light mode
    act(() => colorSchemeValue.setForcedColorScheme("light"));
    expect(getByTestId("color-scheme-display").textContent).toBe("light");

    // Remove forced scheme
    act(() => colorSchemeValue.setForcedColorScheme(null));
    expect(getByTestId("color-scheme-display").textContent).toBe("dark");
  });

  it("updates root dataset when updateRootDataset is true", () => {
    MediaQueryMock.setMatchingQueries("(prefers-color-scheme: dark)");
    let colorSchemeValue: ColorSchemeContextType;

    render(
      <ColorSchemeProvider updateRootDataset>
        <TestComponent
          onMount={(values) => {
            colorSchemeValue = values;
          }}
        />
      </ColorSchemeProvider>,
    );

    expect(document.documentElement.dataset.colorScheme).toBe("dark");

    // Force light mode
    act(() => colorSchemeValue.setForcedColorScheme("light"));
    expect(document.documentElement.dataset.colorScheme).toBe("light");
  });

  it("does not update root dataset when updateRootDataset is false", () => {
    MediaQueryMock.setMatchingQueries("(prefers-color-scheme: dark)");

    render(
      <ColorSchemeProvider updateRootDataset={false}>
        <TestComponent />
      </ColorSchemeProvider>,
    );

    expect(document.documentElement.dataset.colorScheme).toBeUndefined();
  });

  it("cleans up the dataset when unmounted if it was set", () => {
    MediaQueryMock.setMatchingQueries("(prefers-color-scheme: dark)");

    const { unmount } = render(
      <ColorSchemeProvider updateRootDataset>
        <TestComponent />
      </ColorSchemeProvider>,
    );

    expect(document.documentElement.dataset.colorScheme).toBe("dark");

    unmount();

    expect(document.documentElement.dataset.colorScheme).toBeUndefined();
  });
});

describe("useColorScheme", () => {
  let consoleSpy: ReturnType<typeof vi.spyOn> | undefined;

  afterEach(() => {
    consoleSpy?.mockRestore();
  });

  it("does not throw when used outside provider", () => {
    const ComponentWithoutProvider = () => {
      useColorScheme();
      return <div>Test</div>;
    };

    expect(() => {
      render(<ComponentWithoutProvider />);
    }).not.toThrow();
  });

  it("does throws when forcing color scheme outside provider", () => {
    // Suppress console.error for this test
    consoleSpy = vi.spyOn(console, "error").mockImplementation(vi.fn());

    let colorSchemeValue: ColorSchemeContextType;
    render(
      <TestComponent
        onMount={(values) => {
          colorSchemeValue = values;
        }}
      />,
    );

    expect(() => {
      act(() => {
        colorSchemeValue.setForcedColorScheme("dark");
      });
    }).toThrowError(
      "Can only force color scheme when calling useColorScheme inside ColorSchemeProvider",
    );
  });
});

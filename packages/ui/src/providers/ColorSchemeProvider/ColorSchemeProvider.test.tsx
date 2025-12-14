import type { ColorSchemeContextType } from "./context";

import { render, waitFor } from "@testing-library/react";
import { act, useEffect } from "react";
import { afterEach, beforeAll, describe, expect, it, vi } from "vitest";

import ColorSchemeProvider from "./ColorSchemeProvider";
import { useColorScheme } from "./context";

// Helper component to expose context values for testing
const TestComponent = ({ onMount }: { onMount?: (values: ColorSchemeContextType) => void; }) => {
  const values = useColorScheme();

  useEffect(() => {
    onMount?.(values);
  }, [values, onMount]);

  return <div data-testid="color-scheme-display">{values.colorScheme}</div>;
};

class MediaQueryMock {
  private static matchingQueries: string[] = [];
  private static mocks = new Set<WeakRef<MediaQueryMock>>();
  public static setMatchingQueries(...queries: string[]) {
    this.matchingQueries = queries;

    // Notify all existing mocks of the change
    for (const weakRef of this.mocks) {
      const instance = weakRef.deref();
      if (instance === undefined) {
        this.mocks.delete(weakRef);
        continue;
      }

      instance.triggerChangeEvent();
    }
  }

  public static clear() {
    this.matchingQueries = [];
    this.mocks.clear();
  }

  private query: string;
  private isMatching: boolean;
  private listeners = new Map<string, Array<(event: MediaQueryListEvent) => void>>();
  public constructor(query: string) {
    this.query = query;
    this.isMatching = MediaQueryMock.matchingQueries.includes(query);

    // Register this instance for updates as a weak reference
    MediaQueryMock.mocks.add(new WeakRef(this));
  }

  public get matches() {
    return this.isMatching;
  }

  public get media() {
    return this.query;
  }

  public addEventListener(type: string, callback: (event: MediaQueryListEvent) => void) {
    let listeners = this.listeners.get(type);
    if (!listeners) {
      listeners = [];
      this.listeners.set(type, listeners);
    }

    listeners.push(callback);
  }

  public removeEventListener(type: string, callback: (event: MediaQueryListEvent) => void) {
    const listeners = this.listeners.get(type);
    if (listeners) {
      this.listeners.set(
        type,
        listeners.filter((listener) => listener !== callback),
      );
    }
  }

  private triggerChangeEvent() {
    const isMatching = MediaQueryMock.matchingQueries.includes(this.query);
    if (this.isMatching === isMatching) {
      return;
    }

    this.isMatching = isMatching;
    for (const listener of this.listeners.get("change") || []) {
      const event = {
        matches: isMatching,
        media: this.query,
      } as MediaQueryListEvent;
      listener(event);
    }
  }
}

describe("ColorSchemeProvider", () => {
  beforeAll(() => {
    window.matchMedia = (query: string) => {
      return new MediaQueryMock(query) as unknown as MediaQueryList;
    };
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

    // Default to dark
    expect(getByTestId("color-scheme-display").textContent).toBe("light");

    // Change to light
    act(() => MediaQueryMock.setMatchingQueries("(prefers-color-scheme: light)"));
    expect(getByTestId("color-scheme-display").textContent).toBe("light");

    // Change back to dark
    act(() => MediaQueryMock.setMatchingQueries("(prefers-color-scheme: dark)"));
    expect(getByTestId("color-scheme-display").textContent).toBe("dark");
  });

  it("overrides system preferences when forced color scheme is set", async () => {
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

  it("reverts to system preferences when forced color scheme set to null", async () => {
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
    act(() => {
      colorSchemeValue.setForcedColorScheme("light");
    });

    expect(getByTestId("color-scheme-display").textContent).toBe("light");

    // Remove forced scheme
    act(() => {
      colorSchemeValue.setForcedColorScheme(null);
    });

    await waitFor(() => {
      expect(getByTestId("color-scheme-display").textContent).toBe("dark");
    });
  });

  it("updates root dataset when updateRootDataset is true", async () => {
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
    // Suppress console.error for this test
    consoleSpy = vi.spyOn(console, "error").mockImplementation(vi.fn());

    const ComponentWithoutProvider = () => {
      useColorScheme();
      return <div>Test</div>;
    };

    expect(() => {
      render(<ComponentWithoutProvider />);
    }).not.toThrow();
  });
});

import type { FC } from "react";
import type { MotionPreferenceContextType } from "./context";

import { render } from "@testing-library/react";
import { act, useEffect } from "react";
import { afterEach, beforeAll, describe, expect, it, vi } from "vitest";

import { MediaQueryMock } from "../../test-utils/mediaQueryMock";
import { useMotionPreference } from "./context";
import MotionPreferenceProvider from "./MotionPreferenceProvider";

// Helper component to expose context values for testing
type TestComponentProps = {
  onMount?: (values: MotionPreferenceContextType) => void;
};

const TestComponent: FC<TestComponentProps> = ({ onMount }) => {
  const values = useMotionPreference();

  useEffect(() => {
    onMount?.(values);
  }, [values, onMount]);

  return <div data-testid="motion-preference-display">{values.motionPreference}</div>;
};

describe("MotionPreferenceProvider", () => {
  beforeAll(() => {
    MediaQueryMock.installMock();
  });

  afterEach(() => {
    MediaQueryMock.clear();
    vi.clearAllMocks();

    // Clear root dataset before each test
    delete document.documentElement.dataset.motionPreference;
  });

  it("correctly detects system motion preferences", () => {
    const { getByTestId } = render(
      <MotionPreferenceProvider>
        <TestComponent />
      </MotionPreferenceProvider>,
    );

    // Default to full
    expect(getByTestId("motion-preference-display").textContent).toBe("full");

    // Change system preference to reduced
    act(() => MediaQueryMock.setMatchingQueries("(prefers-reduced-motion: reduce)"));
    expect(getByTestId("motion-preference-display").textContent).toBe("reduced");

    // Change system preference back to full
    act(() => MediaQueryMock.setMatchingQueries("(prefers-reduced-motion: no-preference)"));
    expect(getByTestId("motion-preference-display").textContent).toBe("full");
  });

  it("overrides system preferences when motionPreference prop is set", () => {
    MediaQueryMock.setMatchingQueries("(prefers-reduced-motion: reduce)");
    const { getByTestId } = render(
      <MotionPreferenceProvider motionPreference="full">
        <TestComponent />
      </MotionPreferenceProvider>,
    );

    expect(getByTestId("motion-preference-display").textContent).toBe("full");
  });

  it("supports manual off preference through forced setting", () => {
    MediaQueryMock.setMatchingQueries("(prefers-reduced-motion: no-preference)");
    let motionPreferenceValue: MotionPreferenceContextType;

    const { getByTestId } = render(
      <MotionPreferenceProvider>
        <TestComponent
          onMount={(values) => {
            motionPreferenceValue = values;
          }}
        />
      </MotionPreferenceProvider>,
    );

    expect(getByTestId("motion-preference-display").textContent).toBe("full");

    act(() => motionPreferenceValue.setForcedMotionPreference("off"));
    expect(getByTestId("motion-preference-display").textContent).toBe("off");
  });

  it("prefers forced motion preference over motionPreference prop", () => {
    MediaQueryMock.setMatchingQueries("(prefers-reduced-motion: no-preference)");
    let motionPreferenceValue: MotionPreferenceContextType;

    const { getByTestId } = render(
      <MotionPreferenceProvider motionPreference="full">
        <TestComponent
          onMount={(values) => {
            motionPreferenceValue = values;
          }}
        />
      </MotionPreferenceProvider>,
    );

    expect(getByTestId("motion-preference-display").textContent).toBe("full");

    act(() => motionPreferenceValue.setForcedMotionPreference("reduced"));
    expect(getByTestId("motion-preference-display").textContent).toBe("reduced");
  });

  it("reverts to system preferences when forced motion preference set to null", () => {
    MediaQueryMock.setMatchingQueries("(prefers-reduced-motion: reduce)");
    let motionPreferenceValue: MotionPreferenceContextType;

    const { getByTestId } = render(
      <MotionPreferenceProvider>
        <TestComponent
          onMount={(values) => {
            motionPreferenceValue = values;
          }}
        />
      </MotionPreferenceProvider>,
    );

    expect(getByTestId("motion-preference-display").textContent).toBe("reduced");

    act(() => motionPreferenceValue.setForcedMotionPreference("off"));
    expect(getByTestId("motion-preference-display").textContent).toBe("off");

    act(() => motionPreferenceValue.setForcedMotionPreference(null));
    expect(getByTestId("motion-preference-display").textContent).toBe("reduced");
  });

  it("updates root dataset when updateRootDataset is true", () => {
    MediaQueryMock.setMatchingQueries("(prefers-reduced-motion: reduce)");
    let motionPreferenceValue: MotionPreferenceContextType;

    render(
      <MotionPreferenceProvider updateRootDataset>
        <TestComponent
          onMount={(values) => {
            motionPreferenceValue = values;
          }}
        />
      </MotionPreferenceProvider>,
    );

    expect(document.documentElement.dataset.motionPreference).toBe("reduced");

    act(() => motionPreferenceValue.setForcedMotionPreference("full"));
    expect(document.documentElement.dataset.motionPreference).toBe("full");
  });

  it("does not update root dataset when updateRootDataset is false", () => {
    MediaQueryMock.setMatchingQueries("(prefers-reduced-motion: reduce)");

    render(
      <MotionPreferenceProvider updateRootDataset={false}>
        <TestComponent />
      </MotionPreferenceProvider>,
    );

    expect(document.documentElement.dataset.motionPreference).toBeUndefined();
  });

  it("cleans up the dataset when unmounted if it was set", () => {
    MediaQueryMock.setMatchingQueries("(prefers-reduced-motion: reduce)");

    const { unmount } = render(
      <MotionPreferenceProvider updateRootDataset>
        <TestComponent />
      </MotionPreferenceProvider>,
    );

    expect(document.documentElement.dataset.motionPreference).toBe("reduced");

    unmount();

    expect(document.documentElement.dataset.motionPreference).toBeUndefined();
  });
});

describe("useMotionPreference", () => {
  let consoleSpy: ReturnType<typeof vi.spyOn> | undefined;

  afterEach(() => {
    consoleSpy?.mockRestore();
  });

  it("does not throw when used outside provider", () => {
    const ComponentWithoutProvider = () => {
      useMotionPreference();
      return <div>Test</div>;
    };

    expect(() => {
      render(<ComponentWithoutProvider />);
    }).not.toThrow();
  });

  it("throws when forcing motion preference outside provider", () => {
    consoleSpy = vi.spyOn(console, "error").mockImplementation(vi.fn());

    let motionPreferenceValue: MotionPreferenceContextType;
    render(
      <TestComponent
        onMount={(values) => {
          motionPreferenceValue = values;
        }}
      />,
    );

    expect(() => {
      act(() => {
        motionPreferenceValue.setForcedMotionPreference("off");
      });
    }).toThrowError(
      "Can only force motion preference when calling useMotionPreference inside MotionPreferenceProvider",
    );
  });
});

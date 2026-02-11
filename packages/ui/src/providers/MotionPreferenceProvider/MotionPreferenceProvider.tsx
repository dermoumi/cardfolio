import type { FC, PropsWithChildren } from "react";

import { useEffect, useState } from "react";

import { MotionPreferenceContext } from "./context";

export type MotionPreferenceProviderProps = PropsWithChildren<{
  /**
   * Whether or not to update the root element's dataset with the current motion preference.
   */
  updateRootDataset?: boolean;

  /**
   * A forced motion preference to override system preference.
   *
   * The preference forced by setForcedMotionPreference takes precedence over this prop.
   */
  motionPreference?: MotionPreference | null;
}>;

export type MotionPreference = "off" | "reduced" | "full";

/**
 * Provider to manage and provide motion preference information.
 */
const MotionPreferenceProvider: FC<MotionPreferenceProviderProps> = (
  { updateRootDataset, motionPreference, children },
) => {
  const [systemMotionPreference, setSystemMotionPreference] = useState<MotionPreference>("reduced");
  const [forcedMotionPreference, setForcedMotionPreference] = useState<MotionPreference | null>(
    null,
  );

  const effectiveMotionPreference = forcedMotionPreference ?? motionPreference
    ?? systemMotionPreference;

  // Update setSystemMotionPreference when system preference changes
  useEffect(() => {
    const mediaQuery = window.matchMedia("(prefers-reduced-motion: reduce)");

    const handleChange = (event: MediaQueryListEvent) => {
      setSystemMotionPreference(event.matches ? "reduced" : "full");
    };

    setSystemMotionPreference(mediaQuery.matches ? "reduced" : "full");
    mediaQuery.addEventListener("change", handleChange);

    return () => {
      mediaQuery.removeEventListener("change", handleChange);
    };
  }, []);

  // Update root dataset when effectiveMotionPreference changes, if enabled
  useEffect(() => {
    if (!updateRootDataset) {
      return;
    }

    document.documentElement.dataset.motionPreference = effectiveMotionPreference;

    return () => {
      delete document.documentElement.dataset.motionPreference;
    };
  }, [effectiveMotionPreference, updateRootDataset]);

  return (
    <MotionPreferenceContext.Provider
      value={{ motionPreference: effectiveMotionPreference, setForcedMotionPreference }}
    >
      {children}
    </MotionPreferenceContext.Provider>
  );
};

export default MotionPreferenceProvider;

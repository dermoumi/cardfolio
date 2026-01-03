import type { MotionPreference } from "./MotionPreferenceProvider";

import { createContext, useContext } from "react";

export type MotionPreferenceContextType = {
  /**
   * The current effective motion preference.
   */
  motionPreference: MotionPreference;

  /**
   * Function to force a specific motion preference.
   *
   * Passing `null` will revert to the system motion preference.
   */
  setForcedMotionPreference: (preference: MotionPreference | null) => void;
};

export const MotionPreferenceContext = createContext<MotionPreferenceContextType | undefined>(
  undefined,
);

export const useMotionPreference = (): MotionPreferenceContextType => {
  const context = useContext(MotionPreferenceContext);
  if (context) {
    return context;
  }

  return {
    motionPreference: "full",
    setForcedMotionPreference: () => {
      throw new Error(
        "Can only force motion preference when calling useMotionPreference inside MotionPreferenceProvider",
      );
    },
  };
};

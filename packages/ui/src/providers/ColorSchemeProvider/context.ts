import type { ColorScheme } from "./ColorSchemeProvider";

import { createContext, useContext } from "react";

export type ColorSchemeContextType = {
  /**
   * The current effective color scheme.
   */
  colorScheme: ColorScheme;

  /**
   * Function to force a specific color scheme.
   *
   * Passing `null` will revert to the system color scheme.
   */
  setForcedColorScheme: (scheme: ColorScheme | null) => void;
};

export const ColorSchemeContext = createContext<ColorSchemeContextType | undefined>(undefined);

export const useColorScheme = (): ColorSchemeContextType => {
  const context = useContext(ColorSchemeContext);
  if (context) {
    return context;
  }

  return {
    colorScheme: "light",
    setForcedColorScheme: () => {
      throw new Error(
        "Can only force color scheme when calling useColorScheme inside ColorSchemeProvider",
      );
    },
  };
};

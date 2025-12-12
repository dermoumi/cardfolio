import type { ColorScheme } from "./ColorSchemeProvider";

import { createContext, useContext } from "react";

export type ColorSchemeContextType = {
  /**
   * The current effective color scheme.
   */
  colorScheme: ColorScheme;

  /**
   * Function to force a specific color scheme.
   */
  setForcedColorScheme: (scheme: ColorScheme | null) => void;
};

export const ColorSchemeContext = createContext<ColorSchemeContextType | undefined>(undefined);

export const useColorScheme = (): ColorSchemeContextType => {
  const context = useContext(ColorSchemeContext);

  if (!context) {
    throw new Error("useColorScheme must be used within a ColorSchemeProvider");
  }

  return context;
};

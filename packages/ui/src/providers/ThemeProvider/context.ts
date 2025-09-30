import type { Theme } from "./ThemeProvider";

import { createContext, useContext } from "react";

export type ThemeContextType = {
  theme: Theme;
  forceTheme: (theme: Theme | undefined) => void;
};

export const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

export const useTheme = () => {
  const context = useContext(ThemeContext);
  if (!context) {
    throw new Error("useTheme must be used within a ThemeProvider");
  }

  return context;
};

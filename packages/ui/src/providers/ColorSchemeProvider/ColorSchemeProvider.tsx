import type { FC, PropsWithChildren } from "react";

import { useEffect, useState } from "react";

import { ColorSchemeContext } from "./context";

export type ColorSchemeProviderProps = PropsWithChildren<{
  /**
   * Whether or not to update the root element's dataset with the current color scheme.
   */
  updateRootDataset?: boolean;
}>;

export type ColorScheme = "light" | "dark";

/**
 * Provider to manage and provide color scheme information.
 */
const ColorSchemeProvider: FC<ColorSchemeProviderProps> = ({ updateRootDataset, children }) => {
  const [systemColorScheme, setSystemColorScheme] = useState<ColorScheme>("light");
  const [forcedColorScheme, setForcedColorScheme] = useState<ColorScheme | null>(null);

  const colorScheme = forcedColorScheme ?? systemColorScheme;

  // Update setSystemColorScheme when system preference changes
  useEffect(() => {
    const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");

    const handleChange = (event: MediaQueryListEvent) => {
      setSystemColorScheme(event.matches ? "dark" : "light");
    };

    setSystemColorScheme(mediaQuery.matches ? "dark" : "light");
    mediaQuery.addEventListener("change", handleChange);

    return () => {
      mediaQuery.removeEventListener("change", handleChange);
    };
  }, []);

  // Update root dataset when effectiveColorScheme changes, if enabled
  useEffect(() => {
    if (!updateRootDataset) {
      return;
    }

    document.documentElement.dataset.colorScheme = colorScheme;

    return () => {
      delete document.documentElement.dataset.colorScheme;
    };
  }, [colorScheme, updateRootDataset]);

  return (
    <ColorSchemeContext.Provider value={{ colorScheme, setForcedColorScheme }}>
      {children}
    </ColorSchemeContext.Provider>
  );
};

export default ColorSchemeProvider;

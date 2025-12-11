import type { FC, PropsWithChildren } from "react";

import { useEffect, useState } from "react";

import { ThemeContext } from "./context";

export type Theme = "light" | "dark";

const ThemeProvider: FC<PropsWithChildren> = ({ children }) => {
  const [systemTheme, setSystemTheme] = useState<Theme>("dark");
  const [forcedTheme, setForcedTheme] = useState<Theme | undefined>(undefined);

  useEffect(() => {
    const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");

    const handleChange = (event: MediaQueryListEvent) => {
      setSystemTheme(event.matches ? "dark" : "light");
    };

    setSystemTheme(mediaQuery.matches ? "dark" : "light");

    mediaQuery.addEventListener("change", handleChange);
    return () => mediaQuery.removeEventListener("change", handleChange);
  }, [setSystemTheme]);

  useEffect(() => {
    document.body.dataset.theme = forcedTheme ?? systemTheme;
  }, [forcedTheme, systemTheme]);

  return (
    <ThemeContext.Provider
      value={{ theme: forcedTheme ?? systemTheme, forceTheme: setForcedTheme }}
    >
      {children}
    </ThemeContext.Provider>
  );
};

export default ThemeProvider;

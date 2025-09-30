import type { FC, PropsWithChildren } from "react";
import type { ScreenSize } from "./context";

import { useEffect, useState } from "react";

import { ScreenSizeContext } from "./context";

const BREAKPOINTS = {
  sm: 640,
  md: 1024,
} as const;

const ScreenSizeProvider: FC<PropsWithChildren> = ({ children }) => {
  const [screenSize, setScreenSize] = useState<ScreenSize>("lg");
  const [forcedSize, setForcedSize] = useState<ScreenSize | undefined>(undefined);

  useEffect(() => {
    const updateSize = () => {
      const width = window.innerWidth;

      const sizeValue = width < BREAKPOINTS.sm ? "sm" : width < BREAKPOINTS.md ? "md" : "lg";

      setScreenSize(sizeValue);
    };

    updateSize();

    window.addEventListener("resize", updateSize);
    return () => window.removeEventListener("resize", updateSize);
  }, [setScreenSize]);

  useEffect(() => {
    document.body.dataset.screenSize = forcedSize ?? screenSize;
  }, [forcedSize, screenSize]);

  return (
    <ScreenSizeContext.Provider value={{ screenSize, forceScreenSize: setForcedSize }}>
      {children}
    </ScreenSizeContext.Provider>
  );
};

export default ScreenSizeProvider;

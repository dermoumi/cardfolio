import { createContext, useContext } from "react";

export type ScreenSize = "sm" | "md" | "lg";

export type ScreenSizeContextType = {
  screenSize: ScreenSize;
  forceScreenSize: (size: ScreenSize | undefined) => void;
};

export const ScreenSizeContext = createContext<ScreenSizeContextType | undefined>(undefined);

export const useScreenSize = () => {
  const context = useContext(ScreenSizeContext);
  if (!context) {
    throw new Error("useScreenSize must be used within a ScreenSizeProvider");
  }

  return context;
};

import { createContext } from "react";

export type SurfaceContextType = {
  headerRef?: HTMLDivElement | null;
};

export const SurfaceContext = createContext<SurfaceContextType | null>(null);

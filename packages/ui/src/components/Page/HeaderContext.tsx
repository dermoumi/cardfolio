import { createContext } from "react";

export type HeaderContextType = {
  toolbarRef?: HTMLDivElement | null;
};

export const HeaderContext = createContext<HeaderContextType | null>(null);

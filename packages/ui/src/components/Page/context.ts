import { createContext, useContext } from "react";

export type PageContextType = {
  registerFab: () => void;
  unregisterFab: () => void;
};

export const PageContext = createContext<PageContextType | undefined>(undefined);

export const usePageContext = () => {
  const context = useContext(PageContext);
  if (!context) {
    throw new Error("usePageContext must be used within a Page component");
  }

  return context;
};

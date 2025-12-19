import type { FC } from "react";
import type { ColorSchemeProviderProps } from "../ColorSchemeProvider";

import ColorSchemeProvider from "../ColorSchemeProvider";

export type UiProviderProps = ColorSchemeProviderProps;

/**
 * Helper provider that wraps all UI-related providers.
 */
const UiProvider: FC<UiProviderProps> = ({ children, updateRootDataset, colorScheme }) => {
  return (
    <ColorSchemeProvider updateRootDataset={updateRootDataset} colorScheme={colorScheme}>
      {children}
    </ColorSchemeProvider>
  );
};

export default UiProvider;

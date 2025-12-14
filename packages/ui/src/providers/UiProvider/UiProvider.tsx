import type { FC, PropsWithChildren } from "react";

import ColorSchemeProvider from "../ColorSchemeProvider";

export type UiProviderProps = PropsWithChildren<{
  /**
   * Whether or not to update the root element's dataset with the current color scheme.
   */
  updateRootDataset?: boolean;
}>;

/**
 * Helper provider that wraps all UI-related providers.
 */
const UiProvider: FC<UiProviderProps> = ({ children, updateRootDataset }) => {
  return (
    <ColorSchemeProvider updateRootDataset={updateRootDataset}>
      {children}
    </ColorSchemeProvider>
  );
};

export default UiProvider;

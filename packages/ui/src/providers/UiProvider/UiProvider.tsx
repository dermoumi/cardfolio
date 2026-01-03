import type { FC } from "react";
import type { ColorSchemeProviderProps } from "../ColorSchemeProvider";
import type { MotionPreferenceProviderProps } from "../MotionPreferenceProvider";

import ColorSchemeProvider from "../ColorSchemeProvider";
import MotionPreferenceProvider from "../MotionPreferenceProvider";

export type UiProviderProps = ColorSchemeProviderProps & MotionPreferenceProviderProps;

/**
 * Helper provider that wraps all UI-related providers.
 */
const UiProvider: FC<UiProviderProps> = ({
  children,
  updateRootDataset,
  colorScheme,
  motionPreference,
}) => {
  return (
    <MotionPreferenceProvider
      updateRootDataset={updateRootDataset}
      motionPreference={motionPreference}
    >
      <ColorSchemeProvider updateRootDataset={updateRootDataset} colorScheme={colorScheme}>
        {children}
      </ColorSchemeProvider>
    </MotionPreferenceProvider>
  );
};

export default UiProvider;

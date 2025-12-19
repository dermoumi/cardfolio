import styles from "./variants.module.css";

export const VARIANT_CLASSES = {
  primary: styles.primary,
  secondary: undefined,
  subtle: styles.subtle,
} as const;

export const SIZE_CLASSES = {
  sm: styles.sizeSm,
  md: undefined,
  lg: styles.sizeLg,
};

export const RADIUS_CLASSES = {
  sm: styles.radiusSm,
  md: styles.radiusMd,
  lg: styles.radiusLg,
  full: undefined,
};

import type { FC, PropsWithChildren } from "react";

import classNames from "classnames";

import styles from "./Stack.module.css";
import StackStretch from "./StackStretch";

export type StackProps = PropsWithChildren<{
  horizontal?: boolean;
  gap?: "none" | "small" | "medium" | "large";
}>;

const GAP_MAP = {
  none: styles.gapNone,
  small: styles.gapSmall,
  medium: styles.gapMedium,
  large: styles.gapLarge,
} as const;

export type StackComponent = FC<StackProps> & {
  Stretch: typeof StackStretch;
};

const Stack: StackComponent = ({ horizontal, gap = "medium", children }) => {
  return (
    <div
      className={classNames(
        styles.stack,
        horizontal ? styles.horizontal : styles.vertical,
        GAP_MAP[gap],
      )}
    >
      {children}
    </div>
  );
};

Stack.Stretch = StackStretch;

export default Stack;

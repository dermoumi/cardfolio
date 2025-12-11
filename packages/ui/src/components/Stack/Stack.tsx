import type { FC, PropsWithChildren } from "react";

import classNames from "classnames";

import styles from "./Stack.module.css";
import StackStretch from "./StackStretch";

const GAP_MAP = {
  none: styles.gapNone,
  xs: styles.gapExtraSmall,
  sm: styles.gapSmall,
  md: styles.gapMedium,
  lg: styles.gapLarge,
  xl: styles.gapExtraLarge,
} as const;

export type StackProps = PropsWithChildren<{
  horizontal?: boolean;
  gap?: keyof typeof GAP_MAP;
}>;

export type StackComponent = FC<StackProps> & {
  Stretch: typeof StackStretch;
};

const Stack: StackComponent = ({ horizontal, gap = "md", children }) => {
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

import type { PropsWithChildren } from "react";

import classNames from "classnames";
import { forwardRef } from "react";

import styles from "./FlexGrow.module.css";

export type FlexGrowProps = PropsWithChildren<{
  /**
   * Additional class names to apply to the grow container.
   */
  className?: string;
}>;

const FlexGrow = forwardRef<HTMLDivElement, FlexGrowProps>(({ className, children }, ref) => {
  return (
    <div ref={ref} className={classNames(styles.flexGrow, className)}>
      {children}
    </div>
  );
});

export default FlexGrow;

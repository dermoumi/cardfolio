import type { PropsWithChildren } from "react";

import classNames from "classnames";
import { forwardRef } from "react";

import styles from "./FlexShrink.module.css";

export type FlexShrinkProps = PropsWithChildren<{
  /**
   * Additional class names to apply to the shrink container.
   */
  className?: string;
}>;

const FlexShrink = forwardRef<HTMLDivElement, FlexShrinkProps>(({ className, children }, ref) => {
  return (
    <div ref={ref} className={classNames(styles.flexShrink, className)}>
      {children}
    </div>
  );
});

export default FlexShrink;

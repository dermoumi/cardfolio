import type { PropsWithChildren } from "react";

import classNames from "classnames";
import { forwardRef } from "react";

import styles from "./Grow.module.css";

export type GrowProps = PropsWithChildren<{
  /**
   * Additional class names to apply to the grow container.
   */
  className?: string;
}>;

const Grow = forwardRef<HTMLDivElement, GrowProps>(({ className, children }, ref) => {
  return (
    <div ref={ref} className={classNames(styles.grow, className)}>
      {children}
    </div>
  );
});

export default Grow;

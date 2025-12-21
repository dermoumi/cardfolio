import type { PropsWithChildren } from "react";

import classNames from "classnames";
import { forwardRef } from "react";

import styles from "./Shrink.module.css";

export type ShrinkProps = PropsWithChildren<{
  /**
   * Additional class names to apply to the shrink container.
   */
  className?: string;
}>;

const Shrink = forwardRef<HTMLDivElement, ShrinkProps>(({ className, children }, ref) => {
  return (
    <div ref={ref} className={classNames(styles.shrink, className)}>
      {children}
    </div>
  );
});

export default Shrink;

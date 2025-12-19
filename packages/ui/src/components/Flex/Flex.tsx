import type { PropsWithChildren } from "react";

import classNames from "classnames";
import { forwardRef } from "react";

import styles from "./Flex.module.css";
import { GAP_CLASSES } from "./variants";

export type FlexProps = PropsWithChildren<{
  /**
   * If the flex container is vertical
   *
   * @default false
   */
  vertical?: boolean;

  /**
   * Gap between flex items.
   *
   * @default "md"
   */
  gap?: keyof typeof GAP_CLASSES;

  /**
   * Class name for the flex container.
   */
  className?: string;
}>;

const Flex = forwardRef<HTMLDivElement, FlexProps>(
  ({ vertical, className, children, gap = "md" }, ref) => {
    return (
      <div
        ref={ref}
        className={classNames(
          styles.flex,
          vertical ? styles.vertical : styles.horizontal,
          GAP_CLASSES[gap],
          className,
        )}
      >
        {children}
      </div>
    );
  },
);

Flex.displayName = "Flex";

export default Flex;

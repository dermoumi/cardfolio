import type { ForwardRefExoticComponent, PropsWithChildren, RefAttributes } from "react";

import classNames from "classnames";
import { forwardRef } from "react";

import styles from "./Flex.module.css";
import Grow from "./Grow";
import Shrink from "./Shrink";
import { GAP_CLASSES } from "./variants";

export type FlexProps = PropsWithChildren<{
  /**
   * If the flex container is vertical
   *
   * @default false
   */
  vertical?: boolean;

  /**
   * If the flex content should stretch
   */
  stretch?: boolean;

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

export type FlexComponent = ForwardRefExoticComponent<FlexProps & RefAttributes<HTMLDivElement>> & {
  Grow: typeof Grow;
  Shrink: typeof Shrink;
};

const Flex = forwardRef<HTMLDivElement, FlexProps>(
  ({ vertical, stretch, className, children, gap = "md" }, ref) => {
    return (
      <div
        ref={ref}
        className={classNames(
          styles.flex,
          vertical ? styles.vertical : styles.horizontal,
          stretch ? styles.stretch : undefined,
          GAP_CLASSES[gap],
          className,
        )}
      >
        {children}
      </div>
    );
  },
) as FlexComponent;

Flex.displayName = "Flex";
Flex.Grow = Grow;
Flex.Shrink = Shrink;

export default Flex;

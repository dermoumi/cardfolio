import type { PropsWithChildren } from "react";

import classNames from "classnames";
import { forwardRef } from "react";

import styles from "./BaseButton.module.css";
import { RADIUS_CLASSES, SIZE_CLASSES, VARIANT_CLASSES } from "./variants";

export type BaseButtonProps = PropsWithChildren<{
  /**
   * Click handler for the button.
   */
  onClick?: () => void;

  /**
   * Whether or not the button is disabled.
   */
  disabled?: boolean;

  /**
   * Style variant of the button.
   *
   * @default "primary"
   */
  variant?: keyof typeof VARIANT_CLASSES;

  /**
   * Size of the button.
   *
   * @default "md"
   */
  size?: keyof typeof SIZE_CLASSES;

  /**
   * Radius style of the button.
   *
   * @default "full"
   */
  radius?: keyof typeof RADIUS_CLASSES;

  /**
   * Class name for the button.
   */
  className?: string;
}>;

const BaseButton = forwardRef<HTMLButtonElement, BaseButtonProps>(
  (
    { onClick, disabled, variant = "primary", size = "md", radius = "full", className, children },
    ref,
  ) => {
    return (
      <button
        ref={ref}
        className={classNames(
          styles.baseButton,
          VARIANT_CLASSES[variant],
          SIZE_CLASSES[size],
          RADIUS_CLASSES[radius],
          className,
        )}
        onClick={onClick}
        disabled={disabled}
      >
        {children}
      </button>
    );
  },
);

BaseButton.displayName = "BaseButton";

export default BaseButton;

import type { FC, PropsWithChildren } from "react";

import classNames from "classnames";

import styles from "./Button.module.css";

const VARIANT_CLASSES = {
  primary: styles.primary,
  subtle: styles.subtle,
};

export type ButtonProps = PropsWithChildren<{
  type?: "button" | "submit";
  onClick?: () => void;
  disabled?: boolean;
  variant?: keyof typeof VARIANT_CLASSES;
}>;

const Button: FC<ButtonProps> = (
  { type = "button", onClick, disabled, variant = "primary", children },
) => {
  return (
    <button
      type={type}
      className={classNames(VARIANT_CLASSES[variant], styles.button)}
      onClick={onClick}
      disabled={disabled}
    >
      {children}
    </button>
  );
};

export default Button;

import type { FC } from "react";
import type { IconName } from "../Icon";

import classNames from "classnames";

import Icon from "../Icon";
import styles from "./Button.module.css";

const VARIANT_CLASSES = {
  primary: styles.primary,
  subtle: styles.subtle,
};

type CommonProps = {
  type?: "button" | "submit";
  onClick?: () => void;
  disabled?: boolean;
  variant?: keyof typeof VARIANT_CLASSES;
  icon?: IconName;
  label?: string;
};

export type ButtonProps =
  | CommonProps & { children: React.ReactNode; }
  | (CommonProps & { icon: IconName; label: string; children?: undefined; });

const Button: FC<ButtonProps> = (
  { type = "button", onClick, disabled, variant = "primary", children, icon, label },
) => {
  return (
    <button
      type={type}
      className={classNames(
        VARIANT_CLASSES[variant],
        styles.button,
        { [styles.iconOnly]: !children },
      )}
      onClick={onClick}
      disabled={disabled}
      aria-label={label}
    >
      {icon && <Icon name={icon} />}
      {children}
    </button>
  );
};

export default Button;

import type { FC, MouseEventHandler } from "react";
import type { IconName, IconSize } from "../Icon";

import classNames from "classnames";

import Icon from "../Icon";
import styles from "./Button.module.css";

const VARIANT_CLASSES = {
  primary: styles.primary,
  secondary: styles.secondary,
  subtle: styles.subtle,
} as const;

const SIZE_CLASSES = {
  sm: styles.small,
  md: styles.medium,
  lg: styles.large,
} as const;

type CommonProps = {
  type?: "button" | "submit";
  onClick?: MouseEventHandler;
  disabled?: boolean;
  variant?: keyof typeof VARIANT_CLASSES;
  icon?: IconName;
  label?: string;
  form?: string;
  size?: IconSize & keyof typeof SIZE_CLASSES;
};

export type ButtonProps =
  | CommonProps & { children: React.ReactNode; }
  | (CommonProps & { icon: IconName; label: string; children?: undefined; });

const Button: FC<ButtonProps> = (
  {
    type = "button",
    onClick,
    disabled,
    variant = "primary",
    children,
    icon,
    label,
    form,
    size = "md",
  },
) => {
  return (
    <button
      type={type}
      className={classNames(
        VARIANT_CLASSES[variant],
        SIZE_CLASSES[size],
        styles.button,
        { [styles.iconOnly]: !children },
      )}
      onClick={onClick}
      disabled={disabled}
      aria-label={label}
      form={form}
    >
      {icon && <Icon name={icon} size={size} />}
      {children}
    </button>
  );
};

export default Button;

import type { FC, PropsWithChildren } from "react";

import styles from "./Button.module.css";

export type ButtonProps = PropsWithChildren<{
  type?: "button" | "submit";
  onClick?: () => void;
  disabled?: boolean;
}>;

const Button: FC<ButtonProps> = ({ type = "button", onClick, disabled, children }) => {
  return (
    <button type={type} className={styles.button} onClick={onClick} disabled={disabled}>
      {children}
    </button>
  );
};

export default Button;

import type { FC, PropsWithChildren } from "react";

import styles from "./Button.module.css";

export type ButtonProps = PropsWithChildren<{
  onClick?: () => void;
  disabled?: boolean;
}>;

const Button: FC<ButtonProps> = ({ onClick, disabled, children }) => {
  return (
    <button className={styles.button} onClick={onClick} disabled={disabled}>
      {children}
    </button>
  );
};

export default Button;

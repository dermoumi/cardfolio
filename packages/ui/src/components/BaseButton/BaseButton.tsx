import type { FC, PropsWithChildren } from "react";

import styles from "./BaseButton.module.css";

export type BaseButtonProps = PropsWithChildren<{
  onClick?: () => void;
  disabled?: boolean;
}>;

const BaseButton: FC<BaseButtonProps> = ({ onClick, disabled, children }) => {
  return (
    <button className={styles.baseButton} onClick={onClick} disabled={disabled}>
      {children}
    </button>
  );
};

export default BaseButton;

import type { FC, PropsWithChildren } from "react";

import styles from "./BaseButton.module.css";

export type BaseButtonProps = PropsWithChildren<{
  /**
   * Click handler for the button.
   */
  onClick?: () => void;
  /**
   * Disables the button if true.
   */
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

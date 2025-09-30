import type { FC } from "react";
import type { ButtonProps } from "../Button";

import Button from "../Button";
import styles from "./FloatingAction.module.css";

const FloatingAction: FC<ButtonProps> = (props) => {
  return (
    <div className={styles.floatingAction}>
      <Button {...props} />
    </div>
  );
};

export default FloatingAction;

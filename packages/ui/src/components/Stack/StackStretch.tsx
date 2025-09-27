import type { FC, PropsWithChildren } from "react";

import styles from "./StackStretch.module.css";

export type StackStretchProps = PropsWithChildren;

const StackStretch: FC<StackStretchProps> = ({ children }) => {
  return <div className={styles.stretch}>{children}</div>;
};

export default StackStretch;

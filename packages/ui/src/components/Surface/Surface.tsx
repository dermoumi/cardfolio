import type { FC, PropsWithChildren } from "react";

import classNames from "classnames";

import styles from "./Surface.module.css";

const VARIANT_MAP = {
  soft: styles.soft,
  outlined: styles.outlined,
  subtle: styles.subtle,
} as const;

export type SurfaceProps = PropsWithChildren<{
  variant?: keyof typeof VARIANT_MAP;
  header?: React.ReactNode;
}>;

const Surface: FC<SurfaceProps> = ({ children, variant = "soft", header }) => {
  return (
    <div className={classNames(styles.surface, VARIANT_MAP[variant])}>
      {header && <div className={styles.header}>{header}</div>}
      <div className={styles.body}>{children}</div>
    </div>
  );
};

export default Surface;

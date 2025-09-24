import type { FC, PropsWithChildren } from "react";

import classNames from "classnames";
import { useState } from "react";

import styles from "./Surface.module.css";
import { SurfaceContext } from "./SurfaceContext";
import SurfaceHeader from "./SurfaceHeader";

export type SurfaceProps = PropsWithChildren<{
  variant?: "soft" | "outlined" | "transparent";
}>;

const VARIANT_MAP = {
  soft: styles.soft,
  outlined: styles.outlined,
  transparent: styles.transparent,
} as const;

export type SurfaceComponent = FC<SurfaceProps> & {
  Header: typeof SurfaceHeader;
};

const Surface: SurfaceComponent = ({ children, variant = "soft" }) => {
  const [headerRef, setHeaderRef] = useState<HTMLDivElement | null>(null);

  return (
    <div className={classNames(styles.surface, VARIANT_MAP[variant])}>
      <div className={styles.header} ref={setHeaderRef} />
      <div className={styles.body}>
        <SurfaceContext.Provider value={{ headerRef }}>
          {children}
        </SurfaceContext.Provider>
      </div>
    </div>
  );
};

Surface.Header = SurfaceHeader;

export default Surface;

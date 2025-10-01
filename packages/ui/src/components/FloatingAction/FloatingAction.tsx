import type { FC } from "react";
import type { ButtonProps } from "../Button";

import { useEffect, useState } from "react";

import Button from "../Button";
import { usePageContext } from "../Page";
import styles from "./FloatingAction.module.css";

export type FloatingActionProps = ButtonProps & {
  scrollAware?: boolean;
};

const FloatingAction: FC<FloatingActionProps> = ({ children, scrollAware = true, ...props }) => {
  const { registerFab, unregisterFab } = usePageContext();
  const [lastScrollY, setLastScrollY] = useState(0);
  const [hideLabel, setHideLabel] = useState(false);

  useEffect(() => {
    registerFab();

    const handleScroll = () => {
      const currentScrollY = window.scrollY;
      const maxScrollY = document.body.scrollHeight - window.innerHeight;

      setHideLabel((currentScrollY > lastScrollY) && (currentScrollY !== maxScrollY));
      setLastScrollY(currentScrollY);
    };

    window.addEventListener("scroll", handleScroll, { passive: true });

    return () => {
      window.removeEventListener("scroll", handleScroll);
      unregisterFab();
    };
  }, [registerFab, unregisterFab, lastScrollY, setLastScrollY, setHideLabel]);

  return (
    <div className={styles.floatingAction}>
      <Button {...props}>
        {hideLabel && scrollAware ? null : children}
      </Button>
    </div>
  );
};

export default FloatingAction;

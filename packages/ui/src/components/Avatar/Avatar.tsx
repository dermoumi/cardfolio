import type { FC } from "react";
import type { IconName } from "../Icon";

import classNames from "classnames";

import Icon from "../Icon";
import styles from "./Avatar.module.css";

const AVATAR_SIZES = {
  sm: styles.small,
  md: styles.medium,
  lg: styles.large,
} as const;

export type AvatarProps = {
  icon: IconName;
  alt: string;
  size?: keyof typeof AVATAR_SIZES;
};

const Avatar: FC<AvatarProps> = ({ icon, alt, size = "md" }) => {
  return (
    <div className={classNames(styles.avatar, AVATAR_SIZES[size])}>
      <Icon name={icon} label={alt} />
    </div>
  );
};

export default Avatar;

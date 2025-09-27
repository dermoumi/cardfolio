import type { FC } from "react";

import styles from "./TextInput.module.css";

export type TextInputProps = {
  name: string;
  value?: string;
  type?: "text" | "password" | "search";
  onChange?: (value: string) => void;
  placeholder?: string;
};

const TextInput: FC<TextInputProps> = ({ value, name, type = "text", onChange, placeholder }) => {
  return (
    <input
      className={styles.textInput}
      name={name}
      type={type}
      value={value}
      onChange={(e) => onChange?.(e.target.value)}
      placeholder={placeholder}
    />
  );
};

export default TextInput;

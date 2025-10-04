import type { FC } from "react";

import styles from "./TextInput.module.css";

export type TextInputProps = {
  name: string;
  value?: string;
  type?: "text" | "password" | "search";
  onChange?: (value: string) => void;
  placeholder?: string;
  required?: boolean;
  form?: string;
};

const TextInput: FC<TextInputProps> = (
  { value, name, type = "text", onChange, placeholder, required, form },
) => {
  return (
    <input
      className={styles.textInput}
      name={name}
      type={type}
      value={value}
      onChange={(e) => onChange?.(e.target.value)}
      placeholder={placeholder}
      required={required}
      form={form}
    />
  );
};

export default TextInput;

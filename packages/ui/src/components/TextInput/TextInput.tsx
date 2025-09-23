import type { FC } from "react";

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
      name={name}
      type={type}
      value={value}
      onChange={(e) => onChange?.(e.target.value)}
      placeholder={placeholder}
    />
  );
};

export default TextInput;

import type { FC } from "react";

export type TextInputProps = {
  value: string;
  type: "text" | "password" | "search";
  onChange: (value: string) => void;
  placeholder?: string;
};

const TextInput: FC<TextInputProps> = ({ value, type, onChange, placeholder }) => {
  return (
    <input
      type={type}
      value={value}
      onChange={(e) => onChange(e.target.value)}
      placeholder={placeholder}
    />
  );
};

export default TextInput;

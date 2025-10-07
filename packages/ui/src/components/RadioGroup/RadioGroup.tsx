import type { FC, PropsWithChildren } from "react";

import { useMemo } from "react";

import RadioGroupContext from "./context";
import Option from "./RadioOption";

export type RadioGroupProps = PropsWithChildren<{
  name: string;
  value: string;
  onChange: (value: string) => void;
}>;

type RadioGroupComponent = FC<RadioGroupProps> & {
  Option: typeof Option;
};

const RadioGroup: RadioGroupComponent = ({ name, value, onChange, children }) => {
  const contextValue = useMemo(() => ({ name, value, onChange }), [name, value, onChange]);

  return (
    <div
      role="radiogroup"
      aria-labelledby={name}
    >
      <RadioGroupContext.Provider value={contextValue}>
        {children}
      </RadioGroupContext.Provider>
    </div>
  );
};

RadioGroup.Option = Option;

export default RadioGroup;

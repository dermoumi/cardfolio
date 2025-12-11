import type { ReactElement, ReactNode } from "react";
import type { RadioOptionProps } from "./RadioOption";

import { useMemo } from "react";

import RadioGroupContext from "./context";
import Option from "./RadioOption";

export type RadioGroupProps<T extends string> = {
  name: string;
  value: T;
  onChange: (value: T) => void;
  children: Array<ReactElement<RadioOptionProps>> | ReactElement<RadioOptionProps>;
};

type RadioGroupComponent = (<T extends string>(props: RadioGroupProps<T>) => ReactNode) & {
  Option: typeof Option;
};

const RadioGroup: RadioGroupComponent = ({ name, value, onChange, children }) => {
  const contextValue = useMemo(() => ({ name, value, onChange }), [name, value, onChange]);

  return (
    <div
      role="radiogroup"
      aria-labelledby={name}
    >
      {/* eslint-disable-next-line @typescript-eslint/no-explicit-any */}
      <RadioGroupContext.Provider value={contextValue as any}>
        {children}
      </RadioGroupContext.Provider>
    </div>
  );
};

RadioGroup.Option = Option;

export default RadioGroup;

import type { Component, FC, ReactNode } from "react";

import { useMemo } from "react";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type ReactComponent = FC<any> | Component<any>;

// eslint-disable-next-line @typescript-eslint/no-empty-object-type
type TupleOf<T> = ({} | [unknown]) & readonly T[];

type SplitChildren<T extends TupleOf<ReactComponent>> = T extends [] ? []
  : T extends [infer _, ...infer R extends readonly ReactComponent[]]
    ? [ReactNode[], ...SplitChildren<R>]
  : never;

/**
 * Splits the children into separate arrays based on the provided components.
 */
export default function useSplit<T extends TupleOf<ReactComponent>>(
  children: ReactNode,
  ...components: T
): [...SplitChildren<T>, ReactNode[]] {
  return useMemo(() => {
    const childrenList = Array.isArray(children) ? children : [children];

    const results = new Map<ReactComponent, ReactNode[]>(
      components.map(component => [component, []]),
    );
    const leftovers: ReactNode[] = [];

    childrenList.forEach(child => {
      const whichComponent = components.find(component =>
        typeof child === "object" && child?.type === component
      );
      if (whichComponent) {
        results.get(whichComponent)?.push(child);
      } else {
        leftovers.push(child);
      }
    });

    const splitResults = results.values() as SplitChildren<T>;
    return [...splitResults, leftovers];
  }, [children, components]);
}

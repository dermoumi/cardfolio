import { createFileRoute } from "@tanstack/react-router";
import { useCallback, useState } from "react";

export const Route = createFileRoute("/")({
  component: App,
});

function App() {
  const [state] = useState("hello");

  const callback = useCallback(() => {
    console.log(state);
  }, []);

  return (
    <div>
      <header>
        <p>
          Edit <code>src/routes/index.tsx</code> and save to reload :D
        </p>
        <a href="https://reactjs.org" target="_blank" rel="noopener noreferrer">
          Learn React?
        </a>
        <a href="#" target="_blank" rel="noopener noreferrer" onClick={callback}>
          Does nothing
        </a>
      </header>
    </div>
  );
}

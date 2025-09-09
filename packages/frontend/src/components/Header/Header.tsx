import { Link } from "@tanstack/react-router";

export default function Header() {
  return (
    <header>
      <nav>
        <Link to="/">Home</Link>
        <Link to="/cards">Cards</Link>
      </nav>
    </header>
  );
}

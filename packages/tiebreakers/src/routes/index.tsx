import { FloatingAction, ListView, Page } from "@cardfolio/ui";
import { createFileRoute } from "@tanstack/react-router";
import { useCallback } from "react";

import TournamentCard from "@/components/TournamentCard";
import { useTournamentStore } from "@/store/tournamentStore";

export const Route = createFileRoute("/")({
  component: App,
});

function App() {
  const tournaments = useTournamentStore((state) => state.tournaments);
  const removeTournament = useTournamentStore((state) => state.removeTournament);
  const navigate = Route.useNavigate();

  const handleNewTournament = useCallback(() => {
    navigate({ to: "/new" });
  }, [navigate]);

  return (
    <Page>
      <Page.Header
        variant="centered"
        title="Tiebreaker Calculator"
        actions={
          <FloatingAction onClick={handleNewTournament} icon="plus" size="lg">New</FloatingAction>
        }
      />
      <Page.Content>
        <ListView>
          {tournaments.map(({ id, name, timestamp }) => {
            const handleClick = () => {
              navigate({ to: `/${id}/` });
            };

            const handleDelete = () => {
              if (window.confirm(`Delete tournament "${name}"? This cannot be undone.`)) {
                removeTournament(id);
              }
            };

            return (
              <TournamentCard
                key={id}
                name={name}
                date={new Date(timestamp)}
                onClick={handleClick}
                onDelete={handleDelete}
              />
            );
          })}
        </ListView>
      </Page.Content>
    </Page>
  );
}

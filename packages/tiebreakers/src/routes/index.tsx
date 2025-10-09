import { FloatingAction, ListView, Page, Stack } from "@cardfolio/ui";
import { createFileRoute } from "@tanstack/react-router";
import { useCallback } from "react";

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
      <Stack>
        <Page.Header
          variant="centered"
          title="Tiebreaker Calculator"
          actions={
            <FloatingAction onClick={handleNewTournament} icon="plus" size="lg">New</FloatingAction>
          }
        />
        <ListView>
          {tournaments.map((t) => {
            const handleClick = () => {
              navigate({ to: `/${t.id}/` });
            };

            const handleDelete = () => {
              if (window.confirm(`Delete tournament "${t.name}"? This cannot be undone.`)) {
                removeTournament(t.id);
              }
            };

            return (
              <ListView.Item
                key={t.id}
                onClick={handleClick}
              >
                <ListView.Action
                  onClick={handleDelete}
                  icon="trash"
                  label="Delete"
                />
                <Stack>
                  <div>{t.name}</div>
                  <div>{new Date(t.timestamp).toLocaleString()}</div>
                </Stack>
              </ListView.Item>
            );
          })}
        </ListView>
      </Stack>
    </Page>
  );
}

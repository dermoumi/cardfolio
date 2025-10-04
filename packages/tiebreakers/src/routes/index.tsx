import type { MouseEventHandler } from "react";

import { Button, ListView, Page, Stack } from "@cardfolio/ui";
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
          title="Tournaments"
          actions={<Button onClick={handleNewTournament}>New</Button>}
        />
        <ListView>
          {tournaments.map((t) => {
            const handleClick: MouseEventHandler = (e) => {
              e.preventDefault();
              navigate({ to: `/${t.id}/` });
            };

            const handleDelete: MouseEventHandler = (e) => {
              e.preventDefault();
              if (!window.confirm(`Delete tournament "${t.name}"? This cannot be undone.`)) return;

              removeTournament(t.id);
            };

            return (
              <ListView.Item
                key={t.id}
                actions={<Button onClick={handleDelete}>Delete</Button>}
                onClick={handleClick}
              >
                {t.name}
              </ListView.Item>
            );
          })}
        </ListView>
      </Stack>
    </Page>
  );
}

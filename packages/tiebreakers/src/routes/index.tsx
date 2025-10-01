import type { FormEvent, MouseEventHandler } from "react";

import { Button, ListView, Page, Stack, Surface, TextInput } from "@cardfolio/ui";
import { createFileRoute } from "@tanstack/react-router";
import { useCallback, useState } from "react";

import { useTournamentStore } from "@/store/tournamentStore";

export const Route = createFileRoute("/")({
  component: App,
});

function App() {
  const tournaments = useTournamentStore((state) => state.tournaments);
  const createTournament = useTournamentStore((state) => state.createTournament);
  const removeTournament = useTournamentStore((state) => state.removeTournament);
  const [name, setName] = useState("");
  const navigate = Route.useNavigate();

  const handleCreateTournament = useCallback((event: FormEvent) => {
    event.preventDefault();

    const id = createTournament(name || "Tournament");
    navigate({ to: `/tournament/${id}/` });
  }, [name, createTournament, navigate]);

  return (
    <Page>
      <Stack>
        <Page.Header title="Tournaments" />
        <Surface>
          <form onSubmit={handleCreateTournament}>
            <Stack horizontal gap="small">
              <Stack.Stretch>
                <TextInput
                  name="name"
                  value={name}
                  onChange={(value) => setName(value)}
                  placeholder="Tournament Name"
                />
              </Stack.Stretch>
              <Button type="submit" icon="network">
                Add tournament
              </Button>
            </Stack>
          </form>
        </Surface>
        <ListView>
          {tournaments.map((t) => {
            const handleDelete: MouseEventHandler = (e) => {
              e.preventDefault();
              if (!window.confirm(`Delete tournament "${t.name}"? This cannot be undone.`)) return;

              removeTournament(t.id);
            };

            return (
              <ListView.Item key={t.id}>
                <Route.Link to={`/tournament/${t.id}/`}>{t.name}</Route.Link> (
                <a href="#" onClick={handleDelete}>delete</a>
                )
              </ListView.Item>
            );
          })}
        </ListView>
      </Stack>
    </Page>
  );
}

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

  const handleCreateTournament = useCallback(() => {
    const id = createTournament(name || "Tournament");
    navigate({ to: `/tournament/${id}/` });
  }, [name, createTournament, navigate]);

  return (
    <Page title="Tournaments">
      <Page.Content>
        <Stack>
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
                <Button type="submit">Add tournament</Button>
              </Stack>
            </form>
          </Surface>
          <ListView>
            {tournaments.map((t) => (
              <ListView.Item key={t.id}>
                <Route.Link to={`/tournament/${t.id}/`}>{t.name}</Route.Link> (
                <a
                  href="#"
                  onClick={(e) => {
                    e.preventDefault();
                    if (!window.confirm(`Delete tournament "${t.name}"? This cannot be undone.`)) {
                      return;
                    }
                    removeTournament(t.id);
                  }}
                >
                  delete
                </a>
                )
              </ListView.Item>
            ))}
          </ListView>
        </Stack>
      </Page.Content>
    </Page>
  );
}

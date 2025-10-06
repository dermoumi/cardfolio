import type { Player } from "@/store/tournamentStore";
import type { FormEvent } from "react";

import {
  Button,
  Checkbox,
  ListView,
  NumberInput,
  Page,
  Stack,
  Surface,
  TextInput,
} from "@cardfolio/ui";
import { createFileRoute } from "@tanstack/react-router";
import { nanoid } from "nanoid";
import { useCallback, useId, useState } from "react";

import { useTournamentStore } from "@/store/tournamentStore";

export const Route = createFileRoute("/new")({
  component: RouteComponent,
});

function RouteComponent() {
  const [name, setName] = useState("");
  const [winPoints, setWinPoints] = useState(3);
  const [drawPoints, setDrawPoints] = useState(1);
  const [lossPoints, setLossPoints] = useState(1);
  const [playerList, setPlayerList] = useState<Array<Player>>([]);
  const [shufflePlayers, setShufflePlayers] = useState(true);

  const playerListFormId = useId();
  const [playerName, setPlayerName] = useState("");

  const createTournament = useTournamentStore((state) => state.createTournament);
  const navigate = Route.useNavigate();

  const handleCreateTournament = useCallback((event: FormEvent) => {
    event.preventDefault();

    const id = createTournament(
      name,
      playerList,
      winPoints,
      drawPoints,
      lossPoints,
      shufflePlayers,
    );

    navigate({ to: `/${id}/`, replace: true });
  }, [
    name,
    createTournament,
    navigate,
    playerList,
    winPoints,
    drawPoints,
    lossPoints,
    shufflePlayers,
  ]);

  const handleAddPlayer = useCallback((event: FormEvent) => {
    event.preventDefault();
    if (!playerName) return;

    setPlayerList((prev) => [...prev, { id: nanoid(), name: playerName }]);
    setPlayerName("");
  }, [playerName]);

  return (
    <>
      <form id={playerListFormId} onSubmit={handleAddPlayer} />
      <form onSubmit={handleCreateTournament}>
        <Page>
          <Page.Header
            title="New tournament"
            navSlot={<Page.BackButton from={Route.fullPath} to="/" />}
            actions={<Button type="submit" icon="save">Save</Button>}
          />
          <Stack>
            <Surface header="Tournament name">
              <Stack>
                <TextInput
                  name="name"
                  value={name}
                  onChange={(value) => setName(value)}
                  placeholder="Tournament Name"
                  required
                />
              </Stack>
            </Surface>
            <Surface header="Players">
              <Stack>
                <Stack horizontal>
                  <Stack.Stretch>
                    <TextInput
                      name="new-player-name"
                      placeholder="Add a player..."
                      value={playerName}
                      onChange={setPlayerName}
                      form={playerListFormId}
                    />
                  </Stack.Stretch>
                  <Button
                    disabled={!playerName}
                    onClick={handleAddPlayer}
                    type="submit"
                    form={playerListFormId}
                    icon="plus"
                  >
                    Add
                  </Button>
                </Stack>
                <ListView>
                  {playerList.map(({ id, name: player }) => {
                    const handleDelete = () => {
                      setPlayerList((prev) => prev.filter((toRemove) => toRemove.id !== id));
                    };

                    return (
                      <ListView.Item key={`${id}`}>
                        <ListView.Action onClick={handleDelete} icon="x" label="Remove" />
                        {player}
                      </ListView.Item>
                    );
                  })}
                </ListView>
              </Stack>
            </Surface>
            <Surface header="Settings">
              <Stack>
                <label>
                  Win points:
                  <NumberInput
                    name="winPoints"
                    value={winPoints}
                    onChange={setWinPoints}
                    min={0}
                    required
                  />
                </label>
                <label>
                  Draw points:
                  <NumberInput
                    name="drawPoints"
                    value={drawPoints}
                    onChange={setDrawPoints}
                    min={0}
                    required
                  />
                </label>
                <label>
                  Loss points:
                  <NumberInput
                    name="lossPoints"
                    value={lossPoints}
                    onChange={setLossPoints}
                    min={0}
                    required
                  />
                </label>
                <label>
                  <Checkbox
                    name="shufflePlayers"
                    checked={shufflePlayers}
                    onChange={setShufflePlayers}
                  />
                  Shuffle players at random for first round
                </label>
              </Stack>
            </Surface>
          </Stack>
        </Page>
      </form>
    </>
  );
}

import { nanoid } from "nanoid";
import { create } from "zustand";
import { persist } from "zustand/middleware";

export type ID = string;
export type Result = "A" | "B" | "draw" | "loss";

type PlayerScore = number;

export type Player = {
  id: ID;
  name: string;
};

export type Match = {
  id: ID;
  playerA: ID;
  playerB?: ID;
  result?: Result;
};

export type Round = {
  id: ID;
  number: number;
  matches: Match[];
};

export type Tournament = {
  id: ID;
  name: string;
  players: Player[];
  rounds: Round[];
  currentRound?: number;
  status: "in-progress" | "top-cut" | "finished";
  config: Config;
  timestamp: number;
};

export type Config = {
  winPoints: number;
  drawPoints: number;
  lossPoints: number;
  shuffleRounds: "skipFirst" | "all" | "none";
};

type Store = {
  tournaments: Tournament[];

  // actions
  createTournament: (name: string, players: Player[], config: Config) => ID;
  removeTournament: (tournamentId: ID) => void;

  viewFirstRound: (tournamentId: ID) => void;
  viewLastRound: (tournamentId: ID) => void;
  viewPrevRound: (tournamentId: ID) => void;
  viewNextRound: (tournamentId: ID) => void;
  isViewingFirstRound: (tournamentId: ID) => boolean;
  isViewingLastRound: (tournamentId: ID) => boolean;

  addResult: (tournamentId: ID, roundId: ID, matchId: ID, result: Result | null) => void;
  nextRound: (tournamentId: ID) => void;
  topCut: (tournamentId: ID, cutTo: number) => void;
};

export function getPlayerWinsLossesDraws(
  player: Player,
  rounds: Round[],
): { wins: number; losses: number; draws: number; } {
  const [wins, losses, draws] = rounds
    .flatMap((round) => round.matches)
    .reduce(
      ([w, l, d], { playerA, playerB, result }) => {
        if (result === "loss") {
          l++;
        } else if (playerA === player.id) {
          if (result === "A") w++;
          else if (result === "B") l++;
          else if (result === "draw") d++;
        } else if (playerB === player.id) {
          if (result === "B") w++;
          else if (result === "A") l++;
          else if (result === "draw") d++;
        }

        return [w, l, d];
      },
      [0, 0, 0],
    );

  return { wins, losses, draws };
}

function getPlayerOpponentsIds(player: Player, rounds: Round[]): ID[] {
  return rounds.flatMap(({ matches }) =>
    matches
      .filter((match) => match.playerA === player.id || match.playerB === player.id)
      .map((match) => (match.playerA === player.id ? match.playerB : match.playerA))
      .filter((id): id is ID => !!id)
  );
}

function getPlayerOpponents(
  player: Player,
  rounds: Round[],
  playerList: Player[],
): Player[] {
  return getPlayerOpponentsIds(player, rounds)
    .map((opponentId) => playerList.find((p) => p.id === opponentId))
    .filter((p): p is Player => !!p);
}

function getPlayerRoundsLost(player: Player, rounds: Round[]): number[] {
  const roundsLost: number[] = [];
  rounds.forEach(({ matches, number }) => {
    matches.forEach(({ result, playerA, playerB }) => {
      if (!result || (playerA !== player.id && playerB !== player.id)) return;

      const isPlayerA = playerA === player.id;
      if (result === "loss" || (result === "B" && isPlayerA) || (result === "A" && !isPlayerA)) {
        roundsLost.push(number);
      }
    });
  });

  return roundsLost;
}

function getTotalPossiblePoints(rounds: Round[], { winPoints }: Config): number {
  return rounds.length * winPoints;
}

function getMatchPoints(player: Player, rounds: Round[], config: Config): number {
  const { winPoints, drawPoints, lossPoints } = config;
  const { wins, draws, losses } = getPlayerWinsLossesDraws(player, rounds);
  return wins * winPoints + draws * drawPoints + losses * lossPoints;
}

function getMWP(player: Player, rounds: Round[], config: Config): number {
  const totalPossiblePoints = getTotalPossiblePoints(rounds, config);
  if (totalPossiblePoints === 0) return 0;

  const matchPoints = getMatchPoints(player, rounds, config);
  return matchPoints / totalPossiblePoints;
}

function getOppMWP(
  player: Player,
  rounds: Round[],
  playerList: Player[],
  config: Config,
): number {
  const opponents = getPlayerOpponents(player, rounds, playerList);
  if (opponents.length === 0) return 0;

  const totalMwp = opponents.reduce((sum, opponent) => sum + getMWP(opponent, rounds, config), 0);
  return totalMwp / opponents.length;
}

function getOppsOppMWP(
  player: Player,
  rounds: Round[],
  playerList: Player[],
  config: Config,
): number {
  const opponents = getPlayerOpponents(player, rounds, playerList);
  if (opponents.length === 0) return 0;

  const totalOowp = opponents.reduce(
    (sum, opponent) => sum + getOppMWP(opponent, rounds, playerList, config),
    0,
  );
  return totalOowp / opponents.length;
}

export function calculatePlayerScore(
  player: Player,
  rounds: Round[],
  playerList: Player[],
  config: Config,
): PlayerScore {
  const matchPoints = getMatchPoints(player, rounds, config);

  const oppMWP = getOppMWP(player, rounds, playerList, config);
  const oppOppsMWP = getOppsOppMWP(player, rounds, playerList, config);
  const roundsLost = getPlayerRoundsLost(player, rounds);
  const sumSqRoundsLost = roundsLost.reduce(
    (sum, roundNumber) => sum + roundNumber * roundNumber,
    0,
  );

  return matchPoints * 1_000_000_000
    + Math.min(Math.round(oppMWP * 1000), 999) * 1_000_000
    + Math.min(Math.round(oppOppsMWP * 1000), 999) * 1_000
    + Math.min(sumSqRoundsLost, 999);
}

function getTimesByed(player: Player, rounds: Round[]): number {
  return rounds.flatMap(({ matches }) =>
    matches.filter((match) => match.playerA === player.id && !match.playerB)
  ).length;
}

function shuffleArray<T>(array: T[]): T[] {
  const arr = [...array];
  for (let i = arr.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    [arr[i], arr[j]] = [arr[j]!, arr[i]!];
  }
  return arr;
}

// Helper to sort players by their score in descending order
function sortPlayersByScore(
  playerList: Player[],
  rounds: Round[],
  config: Config,
  shuffle: boolean = true,
): Player[] {
  // We shuffle to add a tiny degree of randomness to players with the same score
  const players = shuffle ? shuffleArray(playerList) : [...playerList];

  // We first sort by times byed to ensure players with byes are ranked heigher
  return players.sort((a, b) =>
    calculatePlayerScore(b, rounds, playerList, config)
    - calculatePlayerScore(a, rounds, playerList, config)
  );
}

// If we have an odd number of players, we need to give a bye
// Out of the players with the least byes, take the one with the lowest score
function getByePlayer(playersByScore: Player[], rounds: Round[]): Player | undefined {
  if (playersByScore.length % 2 === 0) return undefined;

  const minByes = Math.min(...playersByScore.map(p => getTimesByed(p, rounds)));
  const candidatesForBye = playersByScore.filter(p => getTimesByed(p, rounds) === minByes);
  return candidatesForBye[candidatesForBye.length - 1];
}

// Sorts players by their score string in descending order
// pair adjancent, avoid rematches if possible by searching forward
// If odd number of players, last player gets a bye
function generateSwissPairings(
  playerList: Player[],
  rounds: Round[],
  config: Config,
  shuffle: boolean = true,
): Match[] {
  const matches: Match[] = [];
  const pairedPlayerIds = new Set<ID>();

  const playersByScore = sortPlayersByScore(playerList, rounds, config, shuffle);
  const playerForBye = getByePlayer(playersByScore, rounds);

  // Remove the bype player from the players list
  if (playerForBye) {
    const index = playersByScore.findIndex(p => p.id === playerForBye.id);
    if (index !== -1) playersByScore.splice(index, 1);
  }

  // For each player, find their opponent
  playersByScore.forEach((playerA, i) => {
    if (pairedPlayerIds.has(playerA.id)) return;

    const previousOpponentsIds = getPlayerOpponentsIds(playerA, rounds);
    const potentialOpponents = playersByScore.slice(i + 1).filter(p => !pairedPlayerIds.has(p.id));

    // Try to find an opponent, that hasn't been paired yet and isn't a rematch
    const playerB = potentialOpponents.find(({ id }) => !previousOpponentsIds.includes(id))
      ?? potentialOpponents[0];

    if (!playerB) {
      throw new Error("Failed to find opponent for player");
    }

    matches.push({
      id: nanoid(),
      playerA: playerA.id,
      playerB: playerB.id,
    });
    pairedPlayerIds.add(playerA.id);
    pairedPlayerIds.add(playerB.id);
  });

  // Add the bye match if applicable
  if (playerForBye) {
    matches.push({
      id: nanoid(),
      playerA: playerForBye.id,
      playerB: undefined,
    });
  }

  return matches;
}

// Helper to get the effective current round for a tournament
export function getCurrentRound(tournament: Tournament): number | undefined {
  return tournament.currentRound ?? getLastRoundIndex(tournament);
}

// Helper to find a tournament by ID
export function findTournament(
  tournaments: Tournament[],
  tournamentId: ID,
): Tournament | undefined {
  return tournaments.find(t => t.id === tournamentId);
}

// Helper to get last round index
function getLastRoundIndex({ rounds }: Tournament): number | undefined {
  return rounds.length > 0 ? rounds.length - 1 : undefined;
}

// Type guards for tournament status
function isInProgressStatus(tournament: Tournament): boolean {
  return tournament.status === "in-progress";
}

// Generic helper function to update an item in an array by ID
function updateById<T extends { id: ID; }>(
  items: T[],
  id: ID,
  updater: (item: T) => T,
): T[] {
  return items.map((item) => {
    if (item.id === id) {
      return updater(item);
    }
    return item;
  });
}

export const useTournamentStore = create<Store>()(
  persist(
    (set, get) => ({
      tournaments: [],
      createTournament: (name, players, config) => {
        const firstRound: Round = {
          id: nanoid(),
          number: 1,
          matches: generateSwissPairings(players, [], config, config.shuffleRounds === "all"),
        };

        const newTournament: Tournament = {
          id: nanoid(),
          name,
          players,
          rounds: [firstRound],
          currentRound: 0,
          status: "in-progress",
          config,
          timestamp: +Date.now(),
        };

        set(({ tournaments, ...state }) => ({
          ...state,
          tournaments: [...tournaments, newTournament],
        }));

        return newTournament.id;
      },
      removeTournament: (tournamentId: ID) => {
        set(({ tournaments, ...state }) => ({
          ...state,
          tournaments: tournaments.filter(t => t.id !== tournamentId),
        }));
      },
      viewFirstRound: (tournamentId: ID) => {
        set(({ tournaments }) => ({
          tournaments: updateById(tournaments, tournamentId, (tournament) => ({
            ...tournament,
            currentRound: tournament.rounds.length > 0 ? 0 : undefined,
          })),
        }));
      },
      viewLastRound: (tournamentId: ID) => {
        set(({ tournaments }) => ({
          tournaments: updateById(tournaments, tournamentId, (tournament) => ({
            ...tournament,
            currentRound: getLastRoundIndex(tournament),
          })),
        }));
      },
      viewPrevRound: (tournamentId: ID) => {
        set(({ tournaments }) => ({
          tournaments: updateById(tournaments, tournamentId, (tournament) => {
            const current = getCurrentRound(tournament);

            if (current === undefined || current <= 0) return tournament;

            return {
              ...tournament,
              currentRound: current - 1,
            };
          }),
        }));
      },
      viewNextRound: (tournamentId: ID) => {
        set(({ tournaments }) => ({
          tournaments: updateById(tournaments, tournamentId, (tournament) => {
            const current = getCurrentRound(tournament);

            if (current === undefined || current >= tournament.rounds.length - 1) return tournament;

            return {
              ...tournament,
              currentRound: current + 1,
            };
          }),
        }));
      },
      isViewingFirstRound: (tournamentId: ID) => {
        const tournament = findTournament(get().tournaments, tournamentId);
        if (!tournament) return false;

        const current = getCurrentRound(tournament);
        return current === 0;
      },
      isViewingLastRound: (tournamentId: ID) => {
        const tournament = findTournament(get().tournaments, tournamentId);
        if (!tournament) return false;

        const current = getCurrentRound(tournament);
        return current === getLastRoundIndex(tournament);
      },

      addResult: (tournamentId: ID, roundId: ID, matchId: ID, result: Result | null) => {
        set(({ tournaments }) => ({
          tournaments: updateById(tournaments, tournamentId, ({ rounds, ...tournament }) => ({
            ...tournament,
            rounds: updateById(rounds, roundId, ({ matches, ...round }) => ({
              ...round,
              matches: updateById(matches, matchId, (match) => ({
                ...match,
                result: result ?? undefined,
              })),
            })),
          })),
        }));
      },
      nextRound: (tournamentId: ID) => {
        set(({ tournaments }) => ({
          tournaments: updateById(tournaments, tournamentId, (tournament) => {
            if (!isInProgressStatus(tournament)) return tournament;

            const { players, rounds, config } = tournament;

            const newRound: Round = {
              id: nanoid(),
              number: rounds.length + 1,
              matches: generateSwissPairings(
                players,
                rounds,
                config,
                config.shuffleRounds !== "none",
              ),
            };

            return {
              ...tournament,
              rounds: [...rounds, newRound],
              currentRound: rounds.length,
            };
          }),
        }));
      },
      topCut: (tournamentId: ID, cutTo: number) => {
        // TODO: Handle actual top cut logic
        set(({ tournaments }) => ({
          tournaments: updateById(tournaments, tournamentId, (tournament) => {
            if (!isInProgressStatus(tournament)) return tournament;

            const { players, rounds, config } = tournament;

            const playersByScore = sortPlayersByScore(players, rounds, config);

            const topPlayers = playersByScore.slice(0, cutTo);
            return {
              ...tournament,
              players: topPlayers,
              rounds: [],
              status: "top-cut",
            };
          }),
        }));
      },
    }),
    {
      name: "tiebreakers-storage",
      version: 2510090000,
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      migrate: (persistedState: any, version) => {
        if (version === 0) {
          for (const tournament of persistedState.tournaments) {
            tournament.timestamp ??= +Date.now();
          }
        }

        return persistedState;
      },
    },
  ),
);

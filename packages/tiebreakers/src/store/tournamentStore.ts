import { create } from "zustand";
import { persist } from "zustand/middleware";

export type ID = string;
export type Result = "A" | "B" | "draw";

const DEFAULT_WIN_POINTS = 3;
const DEFAULT_DRAW_POINTS = 1;
const DEFAULT_LOSS_POINTS = 0;

type PlayerScore = string;

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
  matches: Array<Match>;
};

export type Tournament = {
  id: ID;
  name: string;
  players: Array<Player>;
  rounds: Array<Round>;
  currentRound?: number;
  status: "setup" | "in-progress" | "top-cut" | "finished";

  // Settings
  winPoints: number;
  drawPoints: number;
  lossPoints: number;
};

type Store = {
  tournaments: Array<Tournament>;

  // actions
  createTournament: (name: string) => ID;
  removeTournament: (tournamentId: ID) => void;

  viewFirstRound: (tournamentId: ID) => void;
  viewLastRound: (tournamentId: ID) => void;
  viewPrevRound: (tournamentId: ID) => void;
  viewNextRound: (tournamentId: ID) => void;
  isViewingFirstRound: (tournamentId: ID) => boolean;
  isViewingLastRound: (tournamentId: ID) => boolean;

  addPlayer: (tournamentId: ID, playerName: string) => ID;
  renamePlayer: (tournamentId: ID, playerId: ID, newName: string) => void;
  removePlayer: (tournamentId: ID, playerId: ID) => void;
  startTournament: (tournamentId: ID) => void;

  addResult: (tournamentId: ID, roundId: ID, matchId: ID, result: Result | null) => void;
  nextRound: (tournamentId: ID) => void;
  topCut: (tournamentId: ID, cutTo: number) => void;
};

function uuid() {
  return crypto.randomUUID();
}

export function getPlayerWinsLossesDraws(
  tournament: Tournament,
  player: Player,
): { wins: number; losses: number; draws: number; } {
  const [wins, losses, draws] = tournament.rounds
    .flatMap((round) => round.matches)
    .reduce(
      ([w, l, d], { playerA, playerB, result }) => {
        if (playerA === player.id) {
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

function getPlayerOpponentsIds(tournament: Tournament, player: Player): Array<ID> {
  return tournament.rounds.flatMap((round) =>
    round.matches
      .filter(
        (match) => match.playerA === player.id || match.playerB === player.id,
      )
      .map((match) => (match.playerA === player.id ? match.playerB : match.playerA))
      .filter((id): id is ID => !!id)
  );
}

function getPlayerOpponents(tournament: Tournament, player: Player): Array<Player> {
  return getPlayerOpponentsIds(tournament, player)
    .map((opponentId) => tournament.players.find((p) => p.id === opponentId))
    .filter((p): p is Player => !!p);
}

function getPlayerRoundsLost(tournament: Tournament, player: Player): Array<number> {
  const roundsLost: Array<number> = [];
  tournament.rounds.forEach((round) => {
    round.matches.forEach(({ result, playerA, playerB }) => {
      if (result && (playerA === player.id || playerB === player.id)) {
        const isPlayerA = playerA === player.id;
        if ((result === "B" && isPlayerA) || (result === "A" && !isPlayerA)) {
          roundsLost.push(round.number);
        }
      }
    });
  });
  return roundsLost;
}

function getTotalPossiblePoints({ rounds, winPoints = DEFAULT_WIN_POINTS }: Tournament): number {
  return rounds.length * winPoints;
}

function getMatchPoints(tournament: Tournament, player: Player): number {
  const { wins, draws, losses } = getPlayerWinsLossesDraws(tournament, player);
  const {
    winPoints = DEFAULT_WIN_POINTS,
    drawPoints = DEFAULT_DRAW_POINTS,
    lossPoints = DEFAULT_LOSS_POINTS,
  } = tournament;
  return wins * winPoints + draws * drawPoints + losses * lossPoints;
}

function getMWP(tournament: Tournament, player: Player): number {
  const totalPossiblePoints = getTotalPossiblePoints(tournament);
  if (totalPossiblePoints === 0) return 0;

  const matchPoints = getMatchPoints(tournament, player);
  return matchPoints / totalPossiblePoints;
}

function getOppMWP(tournament: Tournament, player: Player): number {
  const opponents = getPlayerOpponents(tournament, player);
  if (opponents.length === 0) return 0;

  const totalMwp = opponents.reduce((sum, opponent) => sum + getMWP(tournament, opponent), 0);
  return totalMwp / opponents.length;
}

function getOppsOppMWP(tournament: Tournament, player: Player): number {
  const opponents = getPlayerOpponents(tournament, player);
  if (opponents.length === 0) return 0;

  const totalOowp = opponents.reduce((sum, opponent) => sum + getOppMWP(tournament, opponent), 0);
  return totalOowp / opponents.length;
}

export function calculatePlayerScore(tournament: Tournament, player: Player): PlayerScore {
  const matchPoints = getMatchPoints(tournament, player);

  const oppMWP = getOppMWP(tournament, player);
  const oppOppsMWP = getOppsOppMWP(tournament, player);
  const roundsLost = getPlayerRoundsLost(tournament, player);
  const sumSqRoundsLost = roundsLost.reduce((sum, rounds) => sum + rounds * rounds, 0);

  const scoreValue = matchPoints * 1_000_000_000
    + Math.min(Math.round(oppMWP * 1000), 999) * 1_000_000
    + Math.min(Math.round(oppOppsMWP * 1000), 999) * 1_000
    + Math.min(sumSqRoundsLost, 999);

  return scoreValue.toString().padStart(11, "0");
}

function shuffleArray<T>(array: Array<T>): Array<T> {
  const arr = [...array];
  for (let i = arr.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    [arr[i], arr[j]] = [arr[j]!, arr[i]!];
  }
  return arr;
}

// Helper to sort players by their score in descending order
function sortPlayersByScore(tournament: Tournament): Array<Player> {
  // We shuffle to add a tiny degree of randomness to players with the same score
  return shuffleArray(tournament.players).sort((a, b) => {
    const scoreA = calculatePlayerScore(tournament, a);
    const scoreB = calculatePlayerScore(tournament, b);
    return scoreB.localeCompare(scoreA);
  });
}

// Sorts players by their score string in descending order
// pair adjancent, avoid rematches if possible by searching forward
// If odd number of players, last player gets a bye
function generateSwissPairings(tournament: Tournament): Array<Match> {
  const playersByScore = sortPlayersByScore(tournament);

  const matches: Array<Match> = [];
  const pairedPlayerIds = new Set<ID>();

  for (let i = 0; i < playersByScore.length; i++) {
    const playerA = playersByScore[i];
    if (!playerA) continue;
    if (pairedPlayerIds.has(playerA.id)) continue;

    const opponents = getPlayerOpponentsIds(tournament, playerA);

    let playerB: Player | undefined;

    // Try to find an opponent
    for (let j = i + 1; j < playersByScore.length; j++) {
      const potentialOpp = playersByScore[j];
      if (!potentialOpp) continue;

      if (!pairedPlayerIds.has(potentialOpp.id) && !opponents.includes(potentialOpp.id)) {
        playerB = potentialOpp;
        break;
      }
    }

    // If no opponent found without rematch, just take the next available player
    if (!playerB) {
      for (let j = i + 1; j < playersByScore.length; j++) {
        const potentialOpponent = playersByScore[j];
        if (!potentialOpponent) continue;

        if (!pairedPlayerIds.has(potentialOpponent.id)) {
          playerB = potentialOpponent;
          break;
        }
      }
    }

    if (playerB) {
      matches.push({
        id: uuid(),
        playerA: playerA.id,
        playerB: playerB.id,
      });
      pairedPlayerIds.add(playerA.id);
      pairedPlayerIds.add(playerB.id);
    } else {
      // Bye
      matches.push({
        id: uuid(),
        playerA: playerA.id,
      });
      pairedPlayerIds.add(playerA.id);
    }
  }

  return matches;
}

// Helper to get the effective current round for a tournament
function getCurrentRound(tournament: Tournament): number | undefined {
  return tournament.currentRound ?? getLastRoundIndex(tournament);
}

// Helper to find a tournament by ID
function findTournament(tournaments: Array<Tournament>, tournamentId: ID): Tournament | undefined {
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
  items: Array<T>,
  id: ID,
  updater: (item: T) => T,
): Array<T> {
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
      createTournament: (name: string) => {
        const newTournament: Tournament = {
          id: uuid(),
          name,
          players: [],
          rounds: [],
          status: "setup",
          winPoints: DEFAULT_WIN_POINTS,
          drawPoints: DEFAULT_DRAW_POINTS,
          lossPoints: DEFAULT_LOSS_POINTS,
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
      addPlayer: (tournamentId: ID, playerName: string) => {
        const newPlayer: Player = {
          id: uuid(),
          name: playerName,
        };
        set(({ tournaments }) => ({
          tournaments: updateById(tournaments, tournamentId, ({ players, ...tournament }) => ({
            ...tournament,
            players: [...players, newPlayer],
          })),
        }));
        return newPlayer.id;
      },
      renamePlayer: (tournamentId: ID, playerId: ID, newName: string) => {
        set(({ tournaments }) => ({
          tournaments: updateById(tournaments, tournamentId, ({ players, ...tournament }) => ({
            ...tournament,
            players: updateById(players, playerId, (player) => ({
              ...player,
              name: newName,
            })),
          })),
        }));
      },
      removePlayer: (tournamentId: ID, playerId: ID) => {
        set(({ tournaments }) => ({
          tournaments: updateById(tournaments, tournamentId, ({ players, ...tournament }) => ({
            ...tournament,
            players: players.filter((p) => p.id !== playerId),
          })),
        }));
      },

      startTournament: (tournamentId: ID) => {
        set(({ tournaments }) => ({
          tournaments: updateById(tournaments, tournamentId, (tournament) => {
            if (tournament.status !== "setup") return tournament;

            const firstRoundMatches = generateSwissPairings(tournament);
            const firstRound: Round = {
              id: uuid(),
              number: 1,
              matches: firstRoundMatches,
            };
            return {
              ...tournament,
              rounds: [firstRound],
              currentRound: 0,
              status: "in-progress" as const,
            };
          }),
        }));
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

            const { rounds } = tournament;

            const newRound: Round = {
              id: uuid(),
              number: rounds.length + 1,
              matches: generateSwissPairings(tournament),
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

            const playersByScore = sortPlayersByScore(tournament);

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
    { name: "tiebreakers-storage" },
  ),
);

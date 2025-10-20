import type { FC } from "react";

import { Avatar, ListView, Stack } from "@cardfolio/ui";

import styles from "./TournamentCard.module.css";

export type TournamentCardProps = {
  name: string;
  date: Date;
  onClick: () => void;
  onDelete: () => void;
};

const TournamentCard: FC<TournamentCardProps> = ({ name, date, onClick, onDelete }) => {
  return (
    <ListView.Item onClick={onClick}>
      <Avatar icon="network" alt="Tournament" />
      <Stack gap="xs">
        <div className={styles.name}>{name}</div>
        <div className={styles.date}>{date.toLocaleString()}</div>
      </Stack>
      <ListView.Action onClick={onDelete} icon="trash" label="Delete" />
    </ListView.Item>
  );
};

export default TournamentCard;

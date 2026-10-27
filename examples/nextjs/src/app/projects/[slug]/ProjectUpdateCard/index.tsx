import { Markdown } from "@/components/Markdown";
import { Reactions } from "@/components/Reactions";
import { RelativeTime } from "@/components/RelativeTime";
import { UserChip } from "@/components/UserChip";
import { HealthBadge } from "../../ProjectHealthBadge";
import type { ProjectUpdateCardFragment } from "./ProjectUpdateCardFragment.graphql";
import classes from "./ProjectUpdateCard.module.css";

/** A status update in the feed of a project. */
export const ProjectUpdateCard: React.FC<{
  update: ProjectUpdateCardFragment;
}> = ({ update }) => (
  <article className={classes.card}>
    <header className={classes.header}>
      <UserChip user={update.author} />
      <HealthBadge health={update.health} />
      <RelativeTime
        date={update.createdAt}
        className={`${classes.time} muted`}
      />
    </header>
    <Markdown source={update.body} />
    <Reactions subjectId={update.id} reactions={update.reactions} />
  </article>
);

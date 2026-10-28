import { RelativeTime } from "@/components/RelativeTime";
import { UserChip } from "@/components/UserChip";
import type { AttachmentKind } from "@/schema/base.graphql";
import type { IssueAttachmentsFragment } from "./IssueAttachmentsFragment.graphql";
import classes from "./IssueAttachments.module.css";

const kindIcons: Record<AttachmentKind, string> = {
  LINK: "🔗",
  PULL_REQUEST: "⤴",
  DESIGN: "🎨",
  DOCUMENT: "📄",
};

/** Links and files attached to an issue. */
export const IssueAttachments: React.FC<{
  issue: IssueAttachmentsFragment;
}> = ({ issue }) => {
  if (issue.attachments.length === 0) {
    return null;
  }
  return (
    <section className="section">
      <h2>Attachments</h2>
      <ul className={classes.attachments}>
        {issue.attachments.map((attachment) => (
          <li key={attachment.id} className={classes.attachment}>
            <span title={attachment.kind}>{kindIcons[attachment.kind]}</span>
            <a
              href={attachment.url}
              target="_blank"
              rel="noreferrer"
              className={classes.title}
            >
              {attachment.title}
            </a>
            <UserChip user={attachment.creator} />
            <RelativeTime date={attachment.createdAt} className="muted" />
          </li>
        ))}
      </ul>
    </section>
  );
};

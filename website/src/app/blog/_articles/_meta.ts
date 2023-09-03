export type ArticleMetadata = {
  slug: string;
  title: string;
  shortDescription: string;
  publishDate: Date;
  render: () => JSX.Element;
};

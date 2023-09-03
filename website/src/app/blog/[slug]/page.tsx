import { notFound } from "next/navigation";
import { articles } from "../_articles/articles";
import { Breadcrumb } from "@/app/_utils/Breadcrumb";
import "../blog.css";
import { Toc } from "@/app/(docs)/_toc";
import { ogp } from "@/app/_utils/metadata";

export function generateStaticParams() {
  return articles.map((article) => ({
    slug: article.slug,
  }));
}

export function generateMetadata({
  params: { slug },
}: {
  params: { slug: string };
}) {
  const article = articles.find((article) => article.slug === slug);
  if (!article) {
    notFound();
  }
  return ogp({
    title: article.title,
    description: article.shortDescription,
  });
}

const dateFormat = new Intl.DateTimeFormat("en-US", {
  dateStyle: "long",
  timeZone: "UTC",
});

export default function BlogPage({
  params: { slug },
}: {
  params: { slug: string };
}) {
  const article = articles.find((article) => article.slug === slug);
  if (!article) {
    notFound();
  }
  return (
    <Toc>
      <main className="blog-article">
        <Breadcrumb
          parents={[
            {
              href: "/blog",
              label: "Blog",
            },
          ]}
          current={article.title}
        />

        <h2>{article.title}</h2>
        <p className="meta">
          Published at {dateFormat.format(article.publishDate)}
        </p>
        {article.render()}
      </main>
    </Toc>
  );
}

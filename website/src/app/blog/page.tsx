import Link from "next/link";
import "./blog.css";
import { articles } from "./_articles/articles";

const dateFormat = new Intl.DateTimeFormat("en-US", {
  dateStyle: "long",
  timeZone: "UTC",
});

export default function BlogTop() {
  return (
    <main className="blog-top">
      <h2>Blog</h2>
      {articles.map((article) => (
        <article key={article.slug}>
          <Link href={`/blog/${article.slug}`}>
            <h3>{article.title}</h3>
          </Link>
          <p>{article.shortDescription}</p>
          <p>
            {dateFormat.format(article.publishDate)}{" "}
            <Link href={`/blog/${article.slug}`}>Read more</Link>
          </p>
        </article>
      ))}
    </main>
  );
}

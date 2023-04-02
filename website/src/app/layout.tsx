import "./globals.css";

const title = "nitrogql documentation";
const description =
  "Documentation of nitrogql, a GraphQL + TypeScript toolchain";

export const metadata = {
  title: {
    template: "%s | nitrogql",
    default: title,
  },
  description,
  openGraph: {
    type: "website",
    title,
    description,
    images: "/nitrogql-logo-and-text.png",
  },
  icons: [{ rel: "icon", url: "/nitrogql-logo.png" }],
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}

import { Outfit } from "next/font/google";
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
};

const font = Outfit({ subsets: ["latin"], variable: "--font-outfit" });

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body className={font.variable}>{children}</body>
    </html>
  );
}

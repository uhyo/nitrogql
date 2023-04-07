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
  openGraph: {
    type: "website",
    title,
    description,
    images: "/nitrogql-logo-and-text-2x1.png",
  },
  twitter: {
    card: "summary_large_image",
    creator: "@uhyo_",
    title,
    description,
    images: "/nitrogql-logo-and-text-2x1.png",
  },
  icons: [{ rel: "icon", url: "/nitrogql-logo.png" }],
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

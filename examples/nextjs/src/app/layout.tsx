import { UrqlProvider } from "@/components/UrqlProvider";
import "./globals.css";

export const metadata = {
  title: "nitrogql + Next.js example",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body>
        <UrqlProvider>{children}</UrqlProvider>
      </body>
    </html>
  );
}

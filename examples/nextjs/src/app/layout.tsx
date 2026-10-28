import { Sidebar } from "@/components/Sidebar";
import { UrqlProvider } from "@/components/UrqlProvider";
import "./globals.css";
import classes from "./layout.module.css";

export const metadata = {
  title: "nitrogql + Next.js example: issue tracker",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body>
        <UrqlProvider>
          <div className={classes.shell}>
            <Sidebar />
            <main className={classes.main}>{children}</main>
          </div>
        </UrqlProvider>
      </body>
    </html>
  );
}

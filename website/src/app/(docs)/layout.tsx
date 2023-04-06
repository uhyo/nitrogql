import { Footer } from "../(utils)/Footer";
import { Header } from "../(utils)/Header";

export default function DocsLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <>
      <Header />
      {children}
      <Footer />
    </>
  );
}

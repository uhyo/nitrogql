import { Footer } from "../_utils/Footer";
import { Header } from "../_utils/Header";
import { Toc } from "./_toc";

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

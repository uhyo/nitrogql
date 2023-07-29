import { Footer } from "../_utils/Footer";
import { Header } from "../_utils/Header";

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

import { Footer } from "../_utils/Footer";
import { Header } from "../_utils/Header";

export const metadata = {
  title: {
    template: "%s | nitrogql blog",
    default: "nitrogql blog",
  },
};

export default function BlogLayout({
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

import Image from "next/image";
import Link from "next/link";
import { Hint } from "@/app/_utils/Hint";
import { Highlight } from "@/app/_utils/Highlight";
import { Figures } from "@/app/_utils/Figures";
import ScreenshotGeneratedTypes from "./figures/screenshot-generated-types.png";
import { Toc } from "../_toc";
import { InPageNav } from "@/app/_utils/InPageNav";

export default function Guides() {
  return (
    <Toc>
      <main>
        <h2>Guides</h2>
        <InPageNav>
          <Link href="/guides/getting-started">Getting Started</Link>
        </InPageNav>
      </main>
    </Toc>
  );
}

import Link from "next/link";
import { Toc } from "../_toc";
import { InPageNav } from "@/app/_utils/InPageNav";

export default function Configuration() {
  return (
    <Toc>
      <main>
        <h2>Configuration</h2>
        <p>
          nitrogql uses a configuration file to specify the location of your
          schema and operations, and to configure how types are generated. The
          configuration file should be placed in the root of your project.
        </p>

        <InPageNav>
          <Link href="/configuration/file-name">Configuration File Name</Link>
          <Link href="/configuration/options">Configuration Options</Link>
        </InPageNav>
      </main>
    </Toc>
  );
}

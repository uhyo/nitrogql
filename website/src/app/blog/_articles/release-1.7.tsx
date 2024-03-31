import { Highlight } from "@/app/_utils/Highlight";
import { ArticleMetadata } from "./_meta";
import Link from "next/link";
import { Hint } from "@/app/_utils/Hint";

export const blogPostRelease1_7: ArticleMetadata = {
  slug: "release-1.7",
  title: "nitrogql 1.7 release: Jest transform, async internals",
  shortDescription: `In nitrogql 1.7, we added a new package named "@nitrogql/jest-transform".
This package is a Jest transformer that lets you load .graphql files from your code during testing.
`,
  publishDate: new Date("2023-03-31T00:00Z"),
  render,
};

function render() {
  return (
    <>
      <p>
        Today, we are happy to announce release of <strong>nitrogql 1.7</strong>
        !
      </p>
      <p>
        <b>nitrogql</b> is a toolchain for using GraphQL in TypeScript projects.
        In 1.7, we added a new package named{" "}
        <code>@nitrogql/jest-transform</code>. This package is a Jest
        transformer that lets you load <code>.graphql</code> files from your
        code during testing. Also, we made some improvements to the internals of
        nitrogql.
      </p>

      <h2 id="jest-transform">Jest transform</h2>
      <p>
        nitrogql recommends importing GraphQL files directly in your code. This
        allows us to maximize type safety and editor support, avoiding all those
        subtle problems that come with mixing GraphQL and TypeScript in out
        file.
      </p>
      <p>
        Previously, this posed a problem when testing the code. We only provided
        Webpack loader and Rollup plugin, which were good for building the
        project, but not for testing with Jest.
      </p>
      <p>
        With the new <code>@nitrogql/jest-transform</code> package, we also
        support this use case. You can now test code that uses GraphQL without
        any hassle.
      </p>

      <p>
        Below is an example of how to use <code>@nitrogql/jest-transform</code>{" "}
        in your Jest configuration:
      </p>
      <Highlight language="js">
        {`{
  "transform": {
    "^.+\\.graphql$": ["@nitrogql/jest-transform", {
      "configFile":  path.resolve(__dirname, "nitrogql.config.js")
    }],
  }
}`}
      </Highlight>

      <p>
        Also, we&apos;ve noticed that there is a need for additional
        transformation after the GraphQL files are loaded. This is because
        nitrogql only transforms GraphQL files to ES modules, and some projects
        need CommonJS support.
      </p>
      <p>
        To address this, we added the <code>additionalTransformer</code> option
        to <code>@nitrogql/jest-transform</code>. You can now apply another
        transformer that can convert ES modules to CommonJS. In practice, your
        Jest configuration might look like this:
      </p>
      <Highlight language="js">
        {`{
  "transform": {
    "^.+\\.graphql$": ["@nitrogql/jest-transform", {
      "configFile":  path.resolve(__dirname, "nitrogql.config.js"),
      "additionalTransformer": ["ts-jest", { isolatedModules: true }],
    }],
  }
}`}
      </Highlight>
      <p>
        We hope that this new package will make it easier for you to test your
        GraphQL code.
      </p>

      <h2 id="async-internals">Async internals</h2>
      <p>
        Since 1.6, we also made some improvements to the internals of nitrogql.
        Now, async Rust is used for parts of the codebase. This change is mostly
        internal, but it allows future improvements to the performance of
        nitrogql.
      </p>
      <p>
        Since nitrogql is written in Rust and compiled to WebAssembly, it should
        be generally fast. However, there is a critically slow feature in
        nitrogql: JavaScript execution. As nitrogql allows using JavaScript for
        configuration file and scalar definitions, it needs to execute
        JavaScript code at runtime. This is done by delegating execution to
        Node.js.
      </p>
      <p>
        For a couple of reasons, using Node.js for this task has been slow.
        First, a separate process is spawned for each execution. Second, the
        Node.js&apos;{" "}
        <a
          href="https://nodejs.org/docs/latest/api/module.html#customization-hooks"
          target="_blank"
          rel="noopener"
        >
          module loading customization hooks
        </a>{" "}
        have extreme overhead.
      </p>
      <p>
        While we cannot eliminate these problems entirely, we can mitigate them
        by making communication with Node.js asynchronous. This is what we did
        in 1.7. Doing so allows us to reuse the same Node.js process for
        multiple executions, reducing the overhead of spawning processes.
      </p>

      <h3 id="syncoronous-nature-of-nodejs-and-wasm">
        Synchronous nature of Node.js and WebAssembly
      </h3>
      <p>
        Currently, execution of a WebAssembly module from Node.js is
        synchronous. That is, JavaScript code and WebAssembly code share the
        same thread and they appear in the same call stack. While there are
        multiple ongoing proposals to bake asynchronous execution into
        WebAssembly, they are not yet available.
      </p>
      <p>
        Within this limitation, we had to make the communication between
        nitrogql and Node.js asynchronous. Particularly, we wanted to use{" "}
        <a
          href="https://doc.rust-lang.org/std/future/trait.Future.html"
          target="_blank"
        >
          Futures
        </a>{" "}
        in Rust code to represent asynchronous communication with Node.js.
      </p>
      <p>
        To achieve this, standard async executors like Tokio and async-std were
        not suitable. Instead, we developed a custom async executor that is
        specifically designed to integrate WebAssembly into Node.js&apos; event
        loop.{" "}
        <a
          href="https://docs.rs/async-executor/latest/async_executor/"
          target="_blank"
        >
          Async-executor
        </a>{" "}
        was a great for building this custom executor on top of. More details
        are available in{" "}
        <a
          href="https://zenn.dev/uhyo/articles/nodejs-wasm-async-communication"
          target="_blank"
        >
          this article
        </a>{" "}
        (Japanese).
      </p>

      <h3 id="conclusion">Conclusion</h3>
      <p>
        nitrogql 1.7 is a release that makes it easier to test your GraphQL code
        with Jest. We hope that you will find this new package useful. Also, we
        made some improvements to the internals of nitrogql, which will allow us
        to improve the performance in the future.
      </p>
      <hr />

      <p>
        <em>
          nitrogql is developed by{" "}
          <a href="https://x.com/uhyo_" target="_blank">
            uhyo
          </a>
          . Contribution is more than welcome!
        </em>
      </p>
    </>
  );
}

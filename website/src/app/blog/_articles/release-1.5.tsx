import { Highlight } from "@/app/_utils/Highlight";
import { ArticleMetadata } from "./_meta";
import Link from "next/link";

export const blogPostRelease1_5: ArticleMetadata = {
  slug: "release-1.5",
  title: "nitrogql 1.5 release: We know you better now, fragments",
  shortDescription: `In nitrogql 1.5, we added an option named "fragmentTypeSuffix" to the nitrogql CLI.
This option allows you to customize the suffix of the name of the type that is generated for each fragment.`,
  publishDate: new Date("2023-12-13T00:00Z"),
  render,
};

function render() {
  return (
    <>
      <p>
        Today, we are happy to announce release of <strong>nitrogql 1.5</strong>
        !
      </p>
      <p>
        <b>nitrogql</b> is a toolchain for using GraphQL in TypeScript projects.
        In 1.5, we added a new option <code>fragmentTypeSuffix</code> to the
        nitrogql CLI. This option allows you to customize the suffix of the name
        of the type that is generated for each fragment.
      </p>

      <h3 id="the-fragmenttypesuffix-option">
        The <code>fragmentTypeSuffix</code> option
      </h3>
      <p>
        Prior to 1.5, nitrogql did not allow you to customize the name of the
        types generated for fragments. The name of the type was always the same
        as the name of the fragment. For example:
      </p>
      <Highlight language="graphql">{`fragment PartialUser on User {
  name
  email
  iconUrl
}`}</Highlight>
      <p>
        The type generated for the above fragment was always{" "}
        <code>PartialUser</code>. In 1.5, you can customize the suffix of the
        type name by using the{" "}
        <Link href="/configuration/options#generate.name.fragmentTypeSuffix">
          <code>fragmentTypeSuffix</code>
        </Link>{" "}
        option. An example configuration is:
      </p>
      <Highlight language="yaml">{`
generate:
  name:
    fragmentTypeSuffix: Fragment
    # ...`}</Highlight>
      <p>
        With the above configuration, the type generated for the fragment above
        will be <code>PartialUserFragment</code>.
      </p>

      <h3 id="why-is-this-useful">Why is this useful?</h3>
      <p>
        Having a meaningful suffix for a specific kind of types is generally
        useful. As such, we&apos;ve already had similar options for other kinds
        of types and variables. Lack of such an option for fragments was our
        oversight.
      </p>
      <p>
        As you might remember, we added support for importing fragments from
        other GraphQL documents in the previous release. It was the beginning of
        the process of enlightening ourselves about Fragments and their use
        cases. This release comes as another step towards that goal.
      </p>
      <p>
        In addition, the ability to customize the suffix of the type name has
        been necessary for migrating from other tools that have different naming
        conventions. Whilst migration is not an easy task, we want to make it as
        easy as possible.
      </p>

      <h3 id="conclusion">Conclusion</h3>
      <p>
        nitrogql 1.5 is a small release that adds a new option to the nitrogql
        CLI. The change was contributed by{" "}
        <a href="https://github.com/tomocrafter" target="_blank">
          tomocrafter
        </a>{" "}
        and we thank him for his contribution!
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

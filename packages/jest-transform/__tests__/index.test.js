import * as query from "./query.graphql";

it("snapshot test", () => {
  expect(query).toMatchSnapshot();
});

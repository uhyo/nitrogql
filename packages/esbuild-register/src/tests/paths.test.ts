import { describe, it, expect } from "vitest";
import { resolvePaths } from "../paths.js";

describe("No wildcard", () => {
  it("should match", () => {
    expect(resolvePaths("foo", { foo: ["bar"] })).toEqual(["bar"]);
  });
  it("should not match", () => {
    expect(resolvePaths("foo", { bar: ["bar"] })).toBe(undefined);
  });
  it("multiple to paths", () => {
    expect(resolvePaths("foo", { foo: ["bar", "baz"] })).toEqual([
      "bar",
      "baz",
    ]);
  });
  it("should not match if not exact", () => {
    expect(resolvePaths("foo/bar", { foo: ["bar"] })).toBe(undefined);
  });
  it("picks correct one from multiple entries", () => {
    expect(
      resolvePaths("foo", {
        pika: ["chu"],
        foo: ["bar"],
      })
    ).toEqual(["bar"]);
  });
});

describe("Wildcard", () => {
  it("should match with wildcard", () => {
    expect(resolvePaths("foo/pikachu", { "foo/*": ["bar/*"] })).toEqual([
      "bar/pikachu",
    ]);
  });
  it("Wildcard in the middle", () => {
    expect(resolvePaths("foo/pikachu/bar", { "foo/*/bar": ["bar/*"] })).toEqual(
      ["bar/pikachu"]
    );
  });
});

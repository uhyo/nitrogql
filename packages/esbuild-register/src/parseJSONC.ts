import { parse, type ParseError } from "jsonc-parser";

export function parseJSONC(source: string): any {
  const errors: ParseError[] = [];
  const result = parse(source, errors);
  if (errors.length > 0) {
    throw new Error("Parse error", {
      cause: new AggregateError(errors),
    });
  }
  return result;
}

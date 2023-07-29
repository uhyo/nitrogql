import { isValidElement } from "react";

const empty: readonly never[] = [];

export function generateTocFromReactNode(
  node: React.ReactNode
): readonly TocItem[] {
  const toc: TocItem[] = [];
  if (typeof node !== "object") {
    return empty;
  }
  if (Array.isArray(node)) {
    return node.flatMap(generateTocFromReactNode);
  }
  if (isValidElement(node)) {
    // fast path
    if (node.type === "p") {
      return empty;
    }
    if (node.type === "h1") {
      return [
        {
          id: node.props.id,
          text: textOfNode(node.props.children),
          level: 1,
        },
      ];
    }
    if (node.type === "h2") {
      return [
        {
          id: node.props.id,
          text: textOfNode(node.props.children),
          level: 2,
        },
      ];
    }
    if (node.type === "h3") {
      return [
        {
          id: node.props.id,
          text: textOfNode(node.props.children),
          level: 3,
        },
      ];
    }
    if (node.type === "h4") {
      return [
        ...toc,
        {
          id: node.props.id,
          text: textOfNode(node.props.children),
          level: 4,
        },
      ];
    }
    if (node.type === "h5") {
      return [
        ...toc,
        {
          id: node.props.id,
          text: textOfNode(node.props.children),
          level: 5,
        },
      ];
    }
    if (node.type === "h6") {
      return [
        ...toc,
        {
          id: node.props.id,
          text: textOfNode(node.props.children),
          level: 6,
        },
      ];
    }
    if (node.props.children) {
      return generateTocFromReactNode(node.props.children);
    }
  }

  return toc;
}

function textOfNode(node: React.ReactNode): string {
  if (typeof node === "string") {
    return node;
  }
  if (typeof node === "number") {
    return String(node);
  }
  if (Array.isArray(node)) {
    return node.map(textOfNode).join("");
  }
  if (isValidElement(node)) {
    return textOfNode(node.props.children);
  }
  return "";
}

export type TocItem = {
  id: string | undefined;
  text: string;
  level: number;
};

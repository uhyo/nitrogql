import { mkdir, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";

/**
 * Creates temporary folder.
 */
export function tmp(): Folder {
  const folder = path.join(tmpdir(), "nitrogql-" + Date.now());
  return new Folder(folder);
}

class Folder {
  #work: Promise<unknown>;
  #path: string;
  constructor(path: string) {
    this.#path = path;
    this.#work = mkdir(path, { recursive: true });
  }

  file(name: string, content: string): this {
    this.#work = this.#work.then(() =>
      writeFile(path.join(this.#path, name), content)
    );
    return this;
  }

  async path(fileName: string = "."): Promise<string> {
    await this.#work;
    return path.join(this.#path, fileName);
  }
}

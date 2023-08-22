import nodeFs from "node:fs";
import path from "node:path";
import { toUtf8 } from "./util.js";

type FD = {
  /**
   * File descriptor (virtual)
   */
  fd: number;
  /**
   * File descriptor (host)
   */
  hostFd?: number;
  /**
   * Path of the file in the virtual file system (already in UTF-8)
   */
  virtualPath: Uint8Array;
  /**
   * Path of the file in the host file system
   */
  hostPath: string;
  /**
   * Whether the file is preopened
   */
  preopen: boolean;
  /**
   * Lookup flags specified when opening the file
   */
  lookupflags: number;
  /**
   * Cache of readdir results.
   */
  readdirCache?: readonly DirEntry[];
};

export type DirEntry = {
  rawEnt: nodeFs.Dirent;
  name: Uint8Array;
  ino: bigint;
};

type FSOptions = {
  preopens: Record<string, string>;
  fs?: typeof import("node:fs");
};

/**
 * File system
 */
export class FileSystem {
  #openedFiles: Map<number, FD> = new Map();
  #nextFd = 0;
  #fs: typeof import("node:fs");
  constructor(options: FSOptions) {
    const { preopens, fs = nodeFs } = options;
    this.#fs = fs;
    let fd = 3;
    for (const [key, value] of Object.entries(preopens)) {
      this.#openedFiles.set(fd, {
        fd,
        virtualPath: toUtf8(key),
        hostPath: path.resolve(value),
        preopen: true,
        lookupflags: 1,
      });
      fd++;
    }
    this.#nextFd = fd;
  }

  /**
   * get file descriptor by fd
   */
  get(fd: number): FD | undefined {
    return this.#openedFiles.get(fd);
  }

  /**
   * Open a file.
   */
  open(
    parentDir: FD,
    relativePath: string,
    lookupflags: number,
    oflags: number
  ): FD {
    const fd = this.#nextFd++;
    const virtualPath = toUtf8(relativePath);
    const hostPath = path.resolve(parentDir.hostPath, relativePath);
    this.#guardRelativeAccess(parentDir, hostPath);
    const hostFd = this.#fs.openSync(
      hostPath,
      translateOFlags(oflags, lookupflags, this.#fs)
    );
    this.#openedFiles.set(fd, {
      fd,
      hostFd,
      virtualPath,
      hostPath,
      preopen: false,
      lookupflags,
    });
    return this.#openedFiles.get(fd)!;
  }

  /**
   * Get stat of a fd.
   */
  stat(fd: FD): nodeFs.BigIntStats {
    if (fd.hostFd === undefined) {
      throw new FSError("Not opened", "badf");
    }
    return this.#fs.fstatSync(fd.hostFd, {
      bigint: true,
    });
  }
  /**
   * Get stat of a file relative to a fd.
   */
  statPath(
    parentDir: FD,
    relativePath: string,
    lookupflags: number
  ): nodeFs.BigIntStats {
    const hostPath = path.resolve(parentDir.hostPath, relativePath);
    this.#guardRelativeAccess(parentDir, hostPath);
    if (lookupflags & 1) {
      // symlink_follow
      return this.#fs.statSync(hostPath, {
        bigint: true,
      });
    } else {
      return this.#fs.lstatSync(hostPath, {
        bigint: true,
      });
    }
  }
  /**
   * Read from a fd.
   */
  readv(fd: FD, buffers: readonly Uint8Array[]): number {
    if (fd.hostFd === undefined) {
      throw new FSError("Not opened", "badf");
    }
    return this.#fs.readvSync(fd.hostFd, buffers);
  }
  /**
   * Write to a fd.
   */
  writev(fd: FD, buffers: readonly Uint8Array[]): number {
    if (fd.hostFd === undefined) {
      throw new FSError("Not opened", "badf");
    }
    return this.#fs.writevSync(fd.hostFd, buffers);
  }
  /**
   * Read directory entries from a fd.
   */
  readdir(fd: FD, noCache: boolean = false): readonly DirEntry[] {
    if (fd.hostFd === undefined) {
      throw new FSError("Not opened", "badf");
    }
    if (fd.readdirCache && !noCache) {
      return fd.readdirCache;
    }
    const entries = this.#fs.readdirSync(fd.hostPath, {
      withFileTypes: true,
    });
    const result = entries.map((e) => ({
      rawEnt: e,
      name: toUtf8(e.name),
      ino: this.#fs.statSync(path.join(fd.hostPath, e.name), {
        bigint: true,
      }).ino,
    }));
    fd.readdirCache = result;
    return result;
  }
  /**
   * Create a directory.
   */
  mkdir(parentDir: FD, relativePath: string): void {
    const hostPath = path.resolve(parentDir.hostPath, relativePath);
    this.#guardRelativeAccess(parentDir, hostPath);
    this.#fs.mkdirSync(hostPath);
  }
  /**
   * Close a fd.
   */
  close(fd: FD): void {
    if (fd.hostFd === undefined) {
      throw new FSError("Not opened", "badf");
    }
    this.#fs.closeSync(fd.hostFd);
    this.#openedFiles.delete(fd.fd);
  }

  /**
   *  Guard against escaping parent directory.
   */
  #guardRelativeAccess(parentDir: FD, hostPath: string): void {
    const rel = path.relative(parentDir.hostPath, hostPath);
    if (rel.startsWith("..")) {
      throw new FSError("Not capable", "notcapable");
    }
  }
}

export type FSErrorKind = "badf" | "notcapable";

export class FSError extends Error {
  constructor(message: string, public kind: FSErrorKind) {
    super(message);
  }
}

function translateOFlags(
  oflags: number,
  lookupflags: number,
  fs: typeof import("node:fs")
): number {
  let result = 0;
  if (oflags & 0x0001) {
    // creat
    result |= fs.constants.O_CREAT;
  }
  if (oflags & 0x0002) {
    // directory
    result |= fs.constants.O_DIRECTORY;
  } else {
    result |= fs.constants.O_RDWR;
  }
  if (oflags & 0x0004) {
    // excl
    result |= fs.constants.O_EXCL;
  }
  if (oflags & 0x0008) {
    // trunc
    result |= fs.constants.O_TRUNC;
  }
  if (!(lookupflags & 0x0001)) {
    // !symlink_follow
    result |= fs.constants.O_SYMLINK;
  }
  return result;
}

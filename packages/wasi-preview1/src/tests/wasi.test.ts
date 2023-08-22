import { Volume, createFsFromVolume } from "memfs";
import { describe, expect, it } from "vitest";
import { initWASI } from "../wasi.js";
import { NestedDirectoryJSON } from "memfs";

// 1 is 64KiB
const memory = new WebAssembly.Memory({ initial: 1 });
const buffer = new DataView(memory.buffer);

describe("args", () => {
  it("args_sizes_get", () => {
    const wasi = initWASI({
      args: ["--foo", "--bar=baz", "qux"],
      env: {},
      preopens: {},
    });
    wasi.setMemory(memory);
    const result = wasi.args_sizes_get(0, 4);
    expect(result).toBe(0);
    // number of args
    expect(buffer.getUint32(0, true)).toBe(3);
    // size of args data
    expect(buffer.getUint32(4, true)).toBe(5 + 9 + 3 + 3);
  });
  it("args_get", () => {
    const wasi = initWASI({
      args: ["--foo", "--bar=baz", "qux"],
      env: {},
      preopens: {},
    });
    wasi.setMemory(memory);
    const result = wasi.args_get(0, 64);
    expect(result).toBe(0);
    const data = readBufAsText(
      new Uint8Array(memory.buffer, 64, 5 + 9 + 3 + 3)
    );
    expect(data).toBe("--foo\0--bar=baz\0qux\0");
    expect(buffer.getUint32(0, true)).toBe(64);
    expect(buffer.getUint32(4, true)).toBe(64 + 6);
    expect(buffer.getUint32(8, true)).toBe(64 + 6 + 10);
  });
});

describe("env", () => {
  it("env_sizes_get", () => {
    const wasi = initWASI({
      args: [],
      env: { FOO: "BAR", HELLO: "WORLD" },
      preopens: {},
    });
    wasi.setMemory(memory);
    const result = wasi.environ_sizes_get(0, 4);
    expect(result).toBe(0);
    // number of env vars
    expect(buffer.getUint32(0, true)).toBe(2);
    // size of env data
    expect(buffer.getUint32(4, true)).toBe(8 + 12);
  });
  it("env_get", () => {
    const wasi = initWASI({
      args: [],
      env: { FOO: "BAR", HELLO: "WORLD" },
      preopens: {},
    });
    wasi.setMemory(memory);
    const result = wasi.environ_get(0, 64);
    expect(result).toBe(0);
    const data = readBufAsText(new Uint8Array(memory.buffer, 64, 8 + 12));
    expect(data).toBe("FOO=BAR\0HELLO=WORLD\0");
    expect(buffer.getUint32(0, true)).toBe(64);
    expect(buffer.getUint32(4, true)).toBe(64 + 8);
  });
});

describe("preopens", () => {
  describe("fd_prestat_get", () => {
    it("should return stats for preopened fd", () => {
      const wasi = initWASI({
        args: [],
        env: {},
        preopens: { "/foo": "/bar" },
      });
      wasi.setMemory(memory);
      const result = wasi.fd_prestat_get(3, 0);
      expect(result).toBe(0);
      // preopentype::dir
      expect(buffer.getUint8(0)).toBe(0);
      // pr_name_len
      expect(buffer.getUint32(4, true)).toBe(4);
    });
    it("should return badfd for non-preopened fd", () => {
      const wasi = initWASI({
        args: [],
        env: {},
        preopens: { "/foo": "/bar" },
      });
      wasi.setMemory(memory);
      const result = wasi.fd_prestat_get(4, 0);
      expect(result).toBe(8);
    });
  });
  describe("fd_prestat_dir_name", () => {
    it("should return name for preopened fd", () => {
      const wasi = initWASI({
        args: [],
        env: {},
        preopens: { "/foo": "/bar" },
      });
      wasi.setMemory(memory);
      const result = wasi.fd_prestat_dir_name(3, 0, 4);
      expect(result).toBe(0);
      const data = readBufAsText(new Uint8Array(memory.buffer, 0, 4));
      expect(data).toBe("/foo");
    });
    it("should return badfd for non-preopened fd", () => {
      const wasi = initWASI({
        args: [],
        env: {},
        preopens: { "/foo": "/bar" },
      });
      wasi.setMemory(memory);
      const result = wasi.fd_prestat_dir_name(4, 0, 4);
      expect(result).toBe(8);
    });
  });
});

describe("fs", () => {
  describe("path_open", () => {
    it("should open a file", () => {
      const fs = createTestFS({
        "/app": {
          "foo.txt": "hello world",
        },
      });
      const wasi = initWASI({
        args: [],
        env: {},
        preopens: { "/": "/app" },
        fs,
      });
      wasi.setMemory(memory);
      writeTextToBuf("foo.txt", memory.buffer);
      const result = wasi.path_open(3, 0, 0, 7, 0, 0, 0, 0, 64);
      expect(result).toBe(0);
      expect(buffer.getUint32(64, true)).toBe(4);
    });
    it("should open a directory when oflags has directory flag", () => {
      const fs = createTestFS({
        "/app": {
          dir: {
            "foo.txt": "hello world",
          },
        },
      });
      const wasi = initWASI({
        args: [],
        env: {},
        preopens: { "/": "/app" },
        fs,
      });
      wasi.setMemory(memory);
      writeTextToBuf("dir", memory.buffer);
      const result = wasi.path_open(3, 0, 0, 3, 2, 0, 0, 0, 64);
      expect(result).toBe(0);
      expect(buffer.getUint32(64, true)).toBe(4);
    });
    it("should return notdir when fd is not a directory and oflags has directory flag", () => {
      const fs = createTestFS({
        "/app": {
          "foo.txt": "hello world",
        },
      });
      const wasi = initWASI({
        args: [],
        env: {},
        preopens: { "/": "/app" },
        fs,
      });
      wasi.setMemory(memory);
      writeTextToBuf("foo.txt", memory.buffer);
      const result = wasi.path_open(3, 0, 0, 7, 2, 0, 0, 0, 64);
      expect(result).toBe(54);
    });
    it("should return noent when path does not exist", () => {
      const fs = createTestFS({
        "/app": {
          "foo.txt": "hello world",
        },
      });
      const wasi = initWASI({
        args: [],
        env: {},
        preopens: { "/": "/app" },
        fs,
      });
      wasi.setMemory(memory);
      writeTextToBuf("bar.txt", memory.buffer);
      const result = wasi.path_open(3, 0, 0, 7, 0, 0, 0, 0, 64);
      expect(result).toBe(44);
    });
    it("should create file when oflags has create flag", () => {
      const fs = createTestFS({
        "/app": {
          "foo.txt": "hello world",
        },
      });
      const wasi = initWASI({
        args: [],
        env: {},
        preopens: { "/": "/app" },
        fs,
      });
      wasi.setMemory(memory);
      writeTextToBuf("bar.txt", memory.buffer);
      const result = wasi.path_open(3, 0, 0, 7, 1, 0, 0, 0, 64);
      expect(result).toBe(0);
      expect(buffer.getUint32(64, true)).toBe(4);
      expect(fs.existsSync("/app/bar.txt")).toBe(true);
    });
    it("should return exist when path exists and oflags has create and excl flags", () => {
      const fs = createTestFS({
        "/app": {
          "foo.txt": "hello world",
        },
      });
      const wasi = initWASI({
        args: [],
        env: {},
        preopens: { "/": "/app" },
        fs,
      });
      wasi.setMemory(memory);
      writeTextToBuf("foo.txt", memory.buffer);
      const result = wasi.path_open(3, 0, 0, 7, 5, 0, 0, 0, 64);
      expect(result).toBe(20);
      expect(fs.readFileSync("/app/foo.txt", { encoding: "utf8" })).toBe(
        "hello world"
      );
    });
    it("should truncate file if oflags has truncate flag", () => {
      const fs = createTestFS({
        "/app": {
          "foo.txt": "hello world",
        },
      });
      const wasi = initWASI({
        args: [],
        env: {},
        preopens: { "/": "/app" },
        fs,
      });
      wasi.setMemory(memory);
      writeTextToBuf("foo.txt", memory.buffer);
      const result = wasi.path_open(3, 0, 0, 7, 9, 0, 0, 0, 64);
      expect(result).toBe(0);
      expect(buffer.getUint32(64, true)).toBe(4);
      expect(fs.readFileSync("/app/foo.txt", { encoding: "utf8" })).toBe("");
    });
  });
  describe("fd_close", () => {
    it("should close file", () => {
      const fs = createTestFS({
        "/app": {
          "foo.txt": "hello world",
        },
      });
      const wasi = initWASI({
        args: [],
        env: {},
        preopens: { "/": "/app" },
        fs,
      });
      writeTextToBuf("foo.txt", memory.buffer);
      wasi.setMemory(memory);
      const result = wasi.path_open(3, 0, 0, 7, 0, 0, 0, 0, 64);
      expect(result).toBe(0);
      const fd = buffer.getUint32(64, true);
      const result2 = wasi.fd_close(fd);
      expect(result2).toBe(0);
      expect(fs.existsSync("/app/foo.txt")).toBe(true);
    });
    it("should return badf when fd is not open", () => {
      const fs = createTestFS({
        "/app": {
          "foo.txt": "hello world",
        },
      });
      const wasi = initWASI({
        args: [],
        env: {},
        preopens: {},
        fs,
      });
      wasi.setMemory(memory);
      const result = wasi.fd_close(4);
      expect(result).toBe(8);
    });
  });
  describe("fd_filestat_get", () => {
    it("should return stats for file", () => {
      const fs = createTestFS({
        "/app": {
          "foo.txt": "hello world",
        },
      });
      const wasi = initWASI({
        args: [],
        env: {},
        preopens: { "/": "/app" },
        fs,
      });
      wasi.setMemory(memory);

      writeTextToBuf("foo.txt", memory.buffer);
      const result = wasi.path_open(3, 0, 0, 7, 0, 0, 0, 0, 64);
      expect(result).toBe(0);
      const fd = buffer.getUint32(64, true);
      const result2 = wasi.fd_filestat_get(fd, 128);
      expect(result2).toBe(0);
      assertFileStat(memory.buffer, 128, 4, 11n);
    });
    it("should return stats for directory", () => {
      const fs = createTestFS({
        "/app": {
          dir: {
            "foo.txt": "hello world",
          },
        },
      });
      const wasi = initWASI({
        args: [],
        env: {},
        preopens: { "/": "/app" },
        fs,
      });
      wasi.setMemory(memory);

      writeTextToBuf("dir", memory.buffer);
      const result = wasi.path_open(3, 0, 0, 3, 2, 0, 0, 0, 64);
      expect(result).toBe(0);

      const fd = buffer.getUint32(64, true);
      const result2 = wasi.fd_filestat_get(fd, 128);
      expect(result2).toBe(0);
      assertFileStat(memory.buffer, 128, 3, 0n);
    });
    // Waiting for https://github.com/streamich/memfs/pull/944
    it.skip("should return stats for symlink", () => {
      const fs = createTestFS({
        "/app": {
          "foo.txt": "hello world",
        },
      });
      fs.linkSync("/app/foo.txt", "/app/bar.txt");

      const wasi = initWASI({
        args: [],
        env: {},
        preopens: { "/": "/app" },
        fs,
      });
      wasi.setMemory(memory);

      writeTextToBuf("bar.txt", memory.buffer);
      const result = wasi.path_open(3, 0, 0, 7, 0, 0, 0, 0, 64);
      expect(result).toBe(0);

      const fd = buffer.getUint32(64, true);
      const result2 = wasi.fd_filestat_get(fd, 128);
      expect(result2).toBe(0);
      assertFileStat(memory.buffer, 128, 7, 11n);
    });
  });
  describe("fd_read", () => {
    // Waiting for https://github.com/streamich/memfs/pull/946
    it.skip("should read from file", () => {
      const fs = createTestFS({
        "/app": {
          "foo.txt": "hello world",
        },
      });
      const wasi = initWASI({
        args: [],
        env: {},
        preopens: { "/": "/app" },
        fs,
      });
      wasi.setMemory(memory);

      writeTextToBuf("foo.txt", memory.buffer, 0);
      const result = wasi.path_open(3, 0, 0, 7, 0, 0, 0, 0, 64);
      expect(result).toBe(0);

      const fd = buffer.getUint32(64, true);
      const { data: iovs, buffers: ioBuffers } = createIovs(
        memory.buffer,
        1024,
        [20]
      );
      new Uint8Array(buffer.buffer).set(iovs, 128);

      const result2 = wasi.fd_read(fd, 128, 1, 136);
      expect(result2).toBe(0);
      expect(buffer.getUint32(136, true)).toBe(11);
      expect(readBufAsText(ioBuffers[0]!)).toBe("hello world");
    });
    it("should return badf when fd is not open", () => {
      const fs = createTestFS({
        "/app": {
          "foo.txt": "hello world",
        },
      });
      const wasi = initWASI({ args: [], env: {}, preopens: {}, fs });
      wasi.setMemory(memory);
      const result = wasi.fd_read(4, 128, 5, 192);
      expect(result).toBe(8);
    });
  });
  describe("fd_write", () => {
    // Waiting for https://github.com/streamich/memfs/pull/946
    it.skip("should write to file", () => {
      const fs = createTestFS({
        "/app": {},
      });
      const wasi = initWASI({
        args: [],
        env: {},
        preopens: {
          "/": "/app",
        },
        fs,
      });
      wasi.setMemory(memory);

      writeTextToBuf("foo.txt", memory.buffer, 0);
      const openResult = wasi.path_open(3, 0, 0, 7, 1, 0, 0, 0, 64);
      expect(openResult).toBe(0);
      const fd = buffer.getUint32(64, true);
      const { data: iovs, buffers: ioBuffers } = createIovs(
        memory.buffer,
        1024,
        [11]
      );
      writeTextToBuf("hello world", ioBuffers[0]!);
      new Uint8Array(buffer.buffer).set(iovs, 128);

      const result = wasi.fd_write(fd, 128, 1, 136);
      expect(result).toBe(0);
      expect(fs.readFileSync("/app", { encoding: "utf8" })).toBe("hello world");
    });
    it("should return badf when fd is not open", () => {
      const fs = createTestFS({
        "/app": {},
      });
      const wasi = initWASI({ args: [], env: {}, preopens: {}, fs });
      wasi.setMemory(memory);
      const result = wasi.fd_write(4, 128, 5, 192);
      expect(result).toBe(8);
    });
  });
  describe("fd_readdir", () => {
    it("should return entries for directory", () => {
      const fs = createTestFS({
        "/app": {
          dir: {
            "foo.txt": "hello world",
            "bar.txt": "hello world",
            "baz.txt": "hello world",
            xxxxxxx: {},
          },
        },
      });
      const wasi = initWASI({
        args: [],
        env: {},
        preopens: {
          "/": "/app",
        },
        fs,
      });
      wasi.setMemory(memory);

      writeTextToBuf("dir", memory.buffer, 0);
      const openResult = wasi.path_open(3, 0, 0, 3, 2, 0, 0, 0, 64);
      expect(openResult).toBe(0);
      const fd = buffer.getUint32(64, true);

      fillBufRandomly(memory.buffer, 1024, 1024);
      const result = wasi.fd_readdir(fd, 1024, 1024, 0n, 256);
      expect(result).toBe(0);

      const writtenSize = buffer.getUint32(256, true);
      expect(writtenSize).toBe((24 + 7) * 4);

      assertDirent(memory.buffer, 1024, 1n, 7, 4);
      expect(readBufAsText(new Uint8Array(memory.buffer, 1024 + 24, 7))).toBe(
        "bar.txt"
      );
      assertDirent(memory.buffer, 1024 + 24 + 7, 2n, 7, 4);
      expect(
        readBufAsText(new Uint8Array(memory.buffer, 1024 + 24 + 7 + 24, 7))
      ).toBe("baz.txt");
      assertDirent(memory.buffer, 1024 + (24 + 7) * 2, 3n, 7, 4);
      expect(
        readBufAsText(
          new Uint8Array(memory.buffer, 1024 + (24 + 7) * 2 + 24, 7)
        )
      ).toBe("foo.txt");
      assertDirent(memory.buffer, 1024 + (24 + 7) * 3, 4n, 7, 3);
      expect(
        readBufAsText(
          new Uint8Array(memory.buffer, 1024 + (24 + 7) * 3 + 24, 3)
        )
      ).toBe("xxx");
    });
    it("should return entries for directory with cookie", () => {
      const fs = createTestFS({
        "/app": {
          dir: {
            "foo.txt": "hello world",
            "bar.txt": "hello world",
            "baz.txt": "hello world",
            xxxxxxx: {},
          },
        },
      });
      const wasi = initWASI({
        args: [],
        env: {},
        preopens: {
          "/": "/app",
        },
        fs,
      });
      wasi.setMemory(memory);

      writeTextToBuf("dir", memory.buffer, 0);
      const openResult = wasi.path_open(3, 0, 0, 3, 2, 0, 0, 0, 64);
      expect(openResult).toBe(0);
      const fd = buffer.getUint32(64, true);

      fillBufRandomly(memory.buffer, 1024, 1024);
      const result = wasi.fd_readdir(fd, 1024, 1024, 2n, 256);
      expect(result).toBe(0);

      const writtenSize = buffer.getUint32(256, true);
      expect(writtenSize).toBe((24 + 7) * 2);

      assertDirent(memory.buffer, 1024, 3n, 7, 4);
      expect(readBufAsText(new Uint8Array(memory.buffer, 1024 + 24, 7))).toBe(
        "foo.txt"
      );
      assertDirent(memory.buffer, 1024 + 24 + 7, 4n, 7, 3);
      expect(
        readBufAsText(new Uint8Array(memory.buffer, 1024 + 24 + 7 + 24, 3))
      ).toBe("xxx");
    });
    it("should fill buffer when buffer is too small", () => {
      const fs = createTestFS({
        "/app": {
          dir: {
            "foo.txt": "hello world",
            "bar.txt": "hello world",
            "baz.txt": "hello world",
            xxxxxxx: {},
          },
        },
      });
      const wasi = initWASI({
        args: [],
        env: {},
        preopens: {
          "/": "/app",
        },
        fs,
      });
      wasi.setMemory(memory);

      writeTextToBuf("dir", memory.buffer, 0);
      const openResult = wasi.path_open(3, 0, 0, 3, 2, 0, 0, 0, 64);
      expect(openResult).toBe(0);
      const fd = buffer.getUint32(64, true);

      fillBufRandomly(memory.buffer, 1024, 1024);
      const result = wasi.fd_readdir(fd, 1024, 88, 0n, 256);
      expect(result).toBe(0);

      const writtenSize = buffer.getUint32(256, true);
      expect(writtenSize).toBe(88);

      assertDirent(memory.buffer, 1024, 1n, 7, 4);
      expect(readBufAsText(new Uint8Array(memory.buffer, 1024 + 24, 7))).toBe(
        "bar.txt"
      );
      assertDirent(memory.buffer, 1024 + 24 + 7, 2n, 7, 4);
      expect(
        readBufAsText(new Uint8Array(memory.buffer, 1024 + 24 + 7 + 24, 7))
      ).toBe("baz.txt");
      assertDirent(memory.buffer, 1024 + (24 + 7) * 2, 3n, 7, 4);
      // foo.txt is truncated
      expect(
        readBufAsText(
          // 88 - (24 + 7) * 2 - 24 = 2
          new Uint8Array(memory.buffer, 1024 + (24 + 7) * 2 + 24, 2)
        )
      ).toBe("fo");
    });
  });
  describe("path_filestat_get", () => {
    it("should return stats for file", () => {
      const fs = createTestFS({
        "/app": {
          "foo.txt": "hello world",
        },
      });
      const wasi = initWASI({
        args: [],
        env: {},
        preopens: {
          "/": "/app",
        },
        fs,
      });
      wasi.setMemory(memory);

      writeTextToBuf("foo.txt", memory.buffer, 0);
      const result = wasi.path_filestat_get(3, 0, 0, 7, 128);
      expect(result).toBe(0);
      assertFileStat(memory.buffer, 128, 4, 11n);
    });
    it("should return stats for directory", () => {
      const fs = createTestFS({
        "/app": {
          dir: {
            "foo.txt": "hello world",
          },
        },
      });
      const wasi = initWASI({
        args: [],
        env: {},
        preopens: {
          "/": "/app",
        },
        fs,
      });
      wasi.setMemory(memory);

      writeTextToBuf("dir", memory.buffer, 0);
      const result = wasi.path_filestat_get(3, 0, 0, 3, 128);
      expect(result).toBe(0);
      assertFileStat(memory.buffer, 128, 3, 0n);
    });
    // Waiting for https://github.com/streamich/memfs/pull/944
    it.skip("should return stats for symlink", () => {
      const fs = createTestFS({
        "/app": {
          "foo.txt": "hello world",
        },
      });
      fs.linkSync("/app/foo.txt", "/app/bar.txt");
      const wasi = initWASI({
        args: [],
        env: {},
        preopens: {
          "/": "/app",
        },
        fs,
      });
      wasi.setMemory(memory);

      writeTextToBuf("bar.txt", memory.buffer, 0);
      const result = wasi.path_filestat_get(3, 0, 0, 7, 128);
      expect(result).toBe(0);
      assertFileStat(memory.buffer, 128, 7, 0n);
    });
    it("should return noent when path does not exist", () => {
      const fs = createTestFS({
        "/app": {
          "foo.txt": "hello world",
        },
      });
      const wasi = initWASI({
        args: [],
        env: {},
        preopens: {
          "/": "/app",
        },
        fs,
      });
      wasi.setMemory(memory);

      writeTextToBuf("bar.txt", memory.buffer, 0);
      const result = wasi.path_filestat_get(3, 0, 0, 7, 128);
      expect(result).toBe(44);
    });
    it("can get stats for nested file", () => {
      const fs = createTestFS({
        "/app": {
          dir: {
            "foo.txt": "hello world",
          },
        },
      });
      const wasi = initWASI({
        args: [],
        env: {},
        preopens: {
          "/": "/app",
        },
        fs,
      });
      wasi.setMemory(memory);

      writeTextToBuf("dir/foo.txt", memory.buffer, 0);
      const result = wasi.path_filestat_get(3, 0, 0, 11, 128);
      expect(result).toBe(0);
      assertFileStat(memory.buffer, 128, 4, 11n);
    });
    it("guards against path traversal", () => {
      const fs = createTestFS({
        "/app": {
          dir: {
            "foo.txt": "hello world",
          },
        },
      });
      const wasi = initWASI({
        args: [],
        env: {},
        preopens: {
          "/": "/app",
        },
        fs,
      });
      wasi.setMemory(memory);

      writeTextToBuf("../foo.txt", memory.buffer, 0);
      const result = wasi.path_filestat_get(3, 0, 0, 11, 128);
      expect(result).toBe(76);
    });
  });
  describe("path_create_directory", () => {
    it("should create directory", () => {
      const fs = createTestFS({
        "/app": {},
      });
      const wasi = initWASI({
        args: [],
        env: {},
        preopens: {
          "/": "/app",
        },
        fs,
      });
      wasi.setMemory(memory);

      writeTextToBuf("dir", memory.buffer, 0);
      const result = wasi.path_create_directory(3, 0, 3);
      expect(result).toBe(0);
      expect(fs.existsSync("/app/dir")).toBe(true);
    });
    it("should return notdir when parent is not a directory", () => {
      const fs = createTestFS({
        "/app": {
          "foo.txt": "hello world",
        },
      });
      const wasi = initWASI({
        args: [],
        env: {},
        preopens: {
          "/": "/app",
        },
        fs,
      });
      wasi.setMemory(memory);

      writeTextToBuf("foo.txt/dir", memory.buffer, 0);
      const result = wasi.path_create_directory(3, 0, 11);
      expect(result).toBe(54);
    });
    it("should return exist when path already exists", () => {
      const fs = createTestFS({
        "/app": {
          dir: {},
        },
      });
      const wasi = initWASI({
        args: [],
        env: {},
        preopens: {
          "/": "/app",
        },
        fs,
      });
      wasi.setMemory(memory);

      writeTextToBuf("dir", memory.buffer, 0);
      const result = wasi.path_create_directory(3, 0, 3);
      expect(result).toBe(20);
    });
  });
});

function readBufAsText(buf: Uint8Array): string {
  const decoder = new TextDecoder();
  return decoder.decode(buf);
}

function writeTextToBuf(text: string, buf: ArrayBuffer, offset?: number): void {
  const encoder = new TextEncoder();
  const encoded = encoder.encode(text);
  new Uint8Array(buf).set(encoded, offset);
}

function fillBufRandomly(
  buf: ArrayBuffer,
  offset: number,
  length: number
): void {
  const view = new DataView(buf, offset);
  for (let i = 0; i < length; i++) {
    view.setUint8(i, Math.floor(Math.random() * 256));
  }
}

function assertFileStat(
  buf: ArrayBuffer,
  offset: number,
  filetype: number,
  size: bigint
) {
  const view = new DataView(buf, offset);
  expect(view.getUint8(16)).toBe(filetype);
  expect(view.getBigUint64(32, true)).toBe(size);
}

function assertDirent(
  buf: ArrayBuffer,
  offset: number,
  d_next: bigint,
  d_namlen: number,
  d_type: number
) {
  const view = new DataView(buf, offset);
  expect(view.getBigUint64(0, true)).toBe(d_next);
  expect(view.getUint32(16, true)).toBe(d_namlen);
  expect(view.getUint8(20)).toBe(d_type);
}

function createIovs(
  memory: ArrayBuffer,
  data_start_ptr: number,
  lengths: readonly number[]
) {
  const view = new DataView(memory);
  let ptr = data_start_ptr;
  const buffers: Uint8Array[] = [];
  for (const [idx, length] of lengths.entries()) {
    view.setUint32(4 * idx, ptr, true);
    buffers.push(new Uint8Array(memory, data_start_ptr, length));
    // align to 8 bytes
    ptr += length + (8 - (length % 8));
  }
  const result = new Uint8Array(8);
  const resultView = new DataView(result.buffer);
  resultView.setUint32(0, data_start_ptr, true);
  resultView.setUint32(4, lengths.length, true);
  return {
    data: result,
    buffers,
  };
}

function createTestFS(json: NestedDirectoryJSON) {
  const vol = new Volume();
  vol.fromNestedJSON(json);
  const fs = createFsFromVolume(vol);
  return fs as unknown as typeof import("node:fs");
}

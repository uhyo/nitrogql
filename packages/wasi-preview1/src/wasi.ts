import crypto from "node:crypto";
import { WASIAPI } from "./api.js";
import { FSError, FileSystem } from "./fs.js";
import * as error from "./error.js";
import {
  generateOneReaddirEntry,
  getFiletypeOfStat,
  writeFilestat,
  writePrestatDir,
} from "./dataStructure.js";
import { fromUtf8, toUtf8, utf8Length, writeBuf } from "./util.js";

export type WASIConfig = {
  /**
   * The command line arguments passed to the WASI module.
   */
  args: readonly string[];
  /**
   * The environment variables passed to the WASI module.
   */
  env: Record<string, string>;
  /**
   * Preopened directories passed to the WASI module.
   * The key is the virtual directory path and the value is the host directory path.
   * The host directory path must be absolute.
   */
  preopens: Record<string, string>;
  /**
   * FS module (for testing)
   */
  fs?: typeof import("node:fs");
};

export type WASIMeta = {
  /**
   * Set the linear memory of the WASI module.
   */
  setMemory: (memory: WebAssembly.Memory) => void;
};

/**
 * Initialize the WASI implementation.
 */
export function initWASI(config: WASIConfig): WASIAPI & WASIMeta {
  const debug = process.env.NODE_DEBUG?.includes("wasi") ?? false;
  const debugLogBuf: string[] = [];
  const debugLog = (message: string) => {
    if (debug) {
      debugLogBuf.push(message);
    }
  };

  let _wasmMemory: WebAssembly.Memory | undefined;
  let _memory: ArrayBuffer | undefined;
  const memory = (): ArrayBuffer => {
    if (!_memory) {
      throw new Error("WASI memory is not set");
    }
    if (_memory.byteLength === 0) {
      // This means that the memory has grown.
      if (!_wasmMemory) {
        throw new Error("WASI memory is not set");
      }
      _memory = _wasmMemory.buffer;
    }
    return _memory;
  };

  const fs = new FileSystem({
    preopens: config.preopens,
    fs: config.fs,
  });

  let functions: WASIAPI & WASIMeta = {
    setMemory: (m) => {
      _wasmMemory = m;
      _memory = m.buffer;
    },
    /**
     * Return a description of the given preopened file descriptor.
     */
    fd_prestat_get: (fd: number, bufPtr: number): number => {
      const fdObj = fs.get(fd);
      if (fdObj === undefined) {
        return error.badf;
      }
      if (!fdObj.preopen) {
        return error.badf;
      }
      const pathLen = fdObj.virtualPath.length;
      writePrestatDir(memory(), bufPtr, pathLen);
      return 0;
    },
    /**
     * Return a description of the given preopened file descriptor.
     */
    fd_prestat_dir_name: (
      fd: number,
      pathPtr: number,
      pathLen: number
    ): number => {
      const fdObj = fs.get(fd);
      if (fdObj === undefined) {
        return error.badf;
      }
      if (!fdObj.preopen) {
        return error.badf;
      }
      writeBuf(memory(), pathPtr, pathLen, fdObj.virtualPath);
      return 0;
    },
    /**
     * Return command-line argument data sizes.
     */
    args_sizes_get: (args_count_buf, args_size_buf): number => {
      const argLen = config.args.length;
      const argSizes = config.args.map((arg) => utf8Length(arg) + 1);
      const bufSize = argSizes.reduce((acc, size) => acc + size, 0);
      const dv = new DataView(memory());
      dv.setUint32(args_count_buf, argLen, true);
      dv.setUint32(args_size_buf, bufSize, true);
      return 0;
    },
    /**
     * Read command-line argument data. The size of the array should match that returned by args_sizes_get. Each argument is expected to be \0 terminated.
     */
    args_get: (argv, argv_buf): number => {
      let currentPtr = argv_buf;
      const dv = new DataView(memory());
      for (const [index, arg] of config.args.entries()) {
        const argUtf8 = toUtf8(arg + "\0");
        writeBuf(memory(), currentPtr, argUtf8.length, argUtf8);
        dv.setUint32(argv + index * 4, currentPtr, true);
        currentPtr += argUtf8.length;
      }
      return 0;
    },
    /**
     * Return environment variable data sizes.
     */
    environ_sizes_get: (
      environ_count_buf: number,
      environ_size_buf: number
    ): number => {
      const envs = Object.entries(config.env);
      const envLen = envs.length;
      const envSizes = envs.map(
        ([key, value]) => utf8Length(key) + utf8Length(value) + 2
      );
      const bufSize = envSizes.reduce((acc, size) => acc + size, 0);
      const dv = new DataView(memory());
      dv.setUint32(environ_count_buf, envLen, true);
      dv.setUint32(environ_size_buf, bufSize, true);
      return 0;
    },
    /**
     * Read environment variable data. The sizes of the buffers should match that returned by environ_sizes_get. Key/value pairs are expected to be joined with =s, and terminated with \0s.
     */
    environ_get: (environ, environ_buf): number => {
      let currentPtr = environ_buf;
      const dv = new DataView(memory());
      for (const [index, [key, value]] of Object.entries(
        config.env
      ).entries()) {
        const dataUtf8 = toUtf8(key + "=" + value + "\0");
        writeBuf(memory(), currentPtr, dataUtf8.length, dataUtf8);
        dv.setUint32(environ + index * 4, currentPtr, true);
        currentPtr += dataUtf8.length;
      }
      return 0;
    },
    /**
     * Open a file or directory. The returned file descriptor is not guaranteed to be the lowest-numbered file descriptor not currently open; it is randomized to prevent applications from depending on making assumptions about indexes, since this is error-prone in multi-threaded contexts. The returned file descriptor is guaranteed to be less than 2**31. Note: This is similar to openat in POSIX.
     */
    path_open: (
      fd: number,
      dirflags: number,
      path_ptr: number,
      path_len: number,
      oflags: number,
      _fs_rights_base: number,
      _fs_rights_inheriting: number,
      fdflags: number,
      ret_buf: number
    ): number => {
      const parentFd = fs.get(fd);
      if (parentFd === undefined) {
        return error.badf;
      }
      const relativePath = fromUtf8(
        new Uint8Array(memory(), path_ptr, path_len)
      );
      debugLog(`${parentFd.virtualPath} / ${relativePath}`);
      try {
        const result = fs.open(parentFd, relativePath, dirflags, oflags);
        debugLog(`fd = ${result.fd}`);
        const dv = new DataView(memory(), ret_buf, 4);
        dv.setUint32(0, result.fd, true);
        return 0;
      } catch (e) {
        return handleFsError(e, debug);
      }
    },
    fd_filestat_get: (fd: number, ret_buf: number): number => {
      const fdObj = fs.get(fd);
      if (fdObj === undefined) {
        return error.badf;
      }
      const stat = fs.stat(fdObj);
      writeFilestat(
        memory(),
        ret_buf,
        stat.dev,
        stat.ino,
        getFiletypeOfStat(stat),
        stat.nlink,
        stat.size,
        // memfs does not support Ns properties
        // See: https://github.com/streamich/memfs/pull/943
        stat.atimeNs ?? 0n,
        stat.mtimeNs ?? 0n,
        stat.ctimeNs ?? 0n
      );
      return 0;
    },
    fd_read: (
      fd: number,
      iovs_ptr: number,
      iovs_len: number,
      ret_buf: number
    ): number => {
      const fdObj = fs.get(fd);
      if (fdObj === undefined) {
        return error.badf;
      }
      const buffers = [];
      const dv = new DataView(memory(), iovs_ptr, 8 * iovs_len);
      for (let i = 0; i < iovs_len; i++) {
        const bufPtr = dv.getUint32(8 * i, true);
        const bufLen = dv.getUint32(8 * i + 4, true);
        buffers.push(new Uint8Array(memory(), bufPtr, bufLen));
      }
      try {
        const result = fs.readv(fdObj, buffers);
        const dv = new DataView(memory(), ret_buf, 4);
        dv.setUint32(0, result, true);
        return 0;
      } catch (e) {
        return handleFsError(e, debug);
      }
    },
    fd_write: (
      fd: number,
      iovs_ptr: number,
      iovs_len: number,
      ret_buf: number
    ): number => {
      const buffers = [];
      const dv = new DataView(memory(), iovs_ptr, 8 * iovs_len);
      for (let i = 0; i < iovs_len; i++) {
        const bufPtr = dv.getUint32(8 * i, true);
        const bufLen = dv.getUint32(8 * i + 4, true);
        buffers.push(new Uint8Array(memory(), bufPtr, bufLen));
      }
      switch (fd) {
        case 1: {
          // stdout
          for (const buffer of buffers) {
            process.stdout.write(buffer);
          }
          const dv = new DataView(memory(), ret_buf, 4);
          dv.setUint32(
            0,
            buffers.reduce((acc, b) => acc + b.length, 0),
            true
          );
          return 0;
        }
        case 2: {
          // stderr
          for (const buffer of buffers) {
            process.stderr.write(buffer);
          }
          const dv = new DataView(memory(), ret_buf, 4);
          dv.setUint32(
            0,
            buffers.reduce((acc, b) => acc + b.length, 0),
            true
          );
          return 0;
        }
      }
      const fdObj = fs.get(fd);
      if (fdObj === undefined) {
        return error.badf;
      }
      try {
        const result = fs.writev(fdObj, buffers);
        const dv = new DataView(memory(), ret_buf, 4);
        dv.setUint32(0, result, true);
        return 0;
      } catch (e) {
        return handleFsError(e, debug);
      }
    },
    fd_fdstat_get: (fd, ret_buf) => {
      throw new Error("Function not implemented.");
    },
    random_get: (buf, buf_len): number => {
      crypto.randomFillSync(new Uint8Array(memory(), buf, buf_len));
      return 0;
    },
    clock_time_get: (id, _precision, ret_buf): number => {
      const dv = new DataView(memory(), ret_buf, 8);
      switch (id) {
        case 0: {
          // realtime
          const nanoseconds = BigInt(Date.now()) * 1_000_000n;
          dv.setBigUint64(0, nanoseconds, true);
          return 0;
        }
        case 1: {
          // monotonic
          dv.setBigUint64(0, process.hrtime.bigint(), true);
          break;
        }
        case 2: {
          // process_cputime_id
          dv.setBigUint64(0, BigInt(process.cpuUsage().user) * 1000n, true);
          break;
        }
        case 3: {
          // thread_cputime_id
          dv.setBigUint64(0, BigInt(process.cpuUsage().user) * 1000n, true);
          break;
        }
      }
      return 0;
    },
    path_filestat_get: (
      fd: number,
      flags: number,
      path_ptr: number,
      path_len: number,
      ret_buf: number
    ): number => {
      const parentFd = fs.get(fd);
      if (parentFd === undefined) {
        return error.badf;
      }
      const relativePath = fromUtf8(
        new Uint8Array(memory(), path_ptr, path_len)
      );
      debugLog(`${parentFd.virtualPath} / ${relativePath}`);
      try {
        const stat = fs.statPath(parentFd, relativePath, flags);
        writeFilestat(
          memory(),
          ret_buf,
          stat.dev,
          stat.ino,
          getFiletypeOfStat(stat),
          stat.nlink,
          stat.size,
          // memfs does not support Ns properties
          // See: https://github.com/streamich/memfs/pull/943
          stat.atimeNs ?? 0n,
          stat.mtimeNs ?? 0n,
          stat.ctimeNs ?? 0n
        );
        return 0;
      } catch (e) {
        return handleFsError(e, debug);
      }
    },
    fd_readdir: (
      fd: number,
      buf: number,
      buf_len: number,
      cookie: bigint,
      ret_buf: number
    ): number => {
      const fdObj = fs.get(fd);
      if (fdObj === undefined) {
        return error.badf;
      }
      try {
        const entries = fs.readdir(fdObj, cookie === 0n).slice(Number(cookie));
        const bufView = new Uint8Array(memory(), buf, buf_len);
        let currentPtr = 0;
        for (const [i, e] of entries.entries()) {
          const oneEntry = generateOneReaddirEntry(e, Number(cookie) + i);
          if (currentPtr + oneEntry.length > buf_len) {
            bufView.set(oneEntry.subarray(0, buf_len - currentPtr), currentPtr);
            currentPtr = buf_len;
            debugLog(`${Number(cookie) + i}: ${fromUtf8(e.name)} (truncated)`);
            break;
          } else {
            bufView.set(oneEntry, currentPtr);
            currentPtr += oneEntry.length;
            debugLog(`${Number(cookie) + i}: ${fromUtf8(e.name)}`);
          }
        }
        const dv = new DataView(memory(), ret_buf, 4);
        dv.setUint32(0, currentPtr, true);
        return 0;
      } catch (e) {
        return handleFsError(e, debug);
      }
    },
    path_create_directory: (
      fd: number,
      path_ptr: number,
      path_len: number
    ): number => {
      const parentFd = fs.get(fd);
      if (parentFd === undefined) {
        return error.badf;
      }
      const relativePath = fromUtf8(
        new Uint8Array(memory(), path_ptr, path_len)
      );
      try {
        fs.mkdir(parentFd, relativePath);
        return 0;
      } catch (e) {
        return handleFsError(e, debug);
      }
    },
    fd_close: (fd: number): number => {
      const fdObj = fs.get(fd);
      if (fdObj === undefined) {
        return error.badf;
      }
      fs.close(fdObj);
      return 0;
    },
    proc_exit: (rval: number): void => {
      process.exit(rval);
    },
    sched_yield: (): number => {
      return 0;
    },
  };

  if (debug) {
    functions = new Proxy(functions, {
      get(target, prop, receiver) {
        const value = Reflect.get(target, prop, receiver);
        if (typeof value === "function") {
          return (...args: unknown[]) => {
            debugLogBuf.length = 0;
            const result = value.apply(target, args);
            console.debug(`${String(prop)}(${args.join(", ")}) = ${result}`);
            if (debugLogBuf.length > 0) {
              console.debug(debugLogBuf.map((x) => `  ${x}`).join("\n"));
            }
            if (result !== 0 && result !== undefined) {
              debugger;
            }
            return result;
          };
        }
        return value;
      },
    });
  }

  return functions;
}

function handleFsError(e: unknown, debug: boolean): number {
  if (e instanceof FSError) {
    switch (e.kind) {
      case "badf": {
        return error.badf;
      }
      case "notcapable": {
        return error.notcapable;
      }
    }
  } else if (e instanceof Error && "code" in e) {
    switch (e.code) {
      case "EEXIST": {
        return error.exist;
      }
      case "ENOENT": {
        return error.noent;
      }
      case "EISDIR": {
        return error.isdir;
      }
      case "ENOTDIR": {
        return error.notdir;
      }
      case "EACCES": {
        return error.acces;
      }
      case "EAGAIN": {
        return error.again;
      }
      case "EBADF": {
        return error.badf;
      }
      default: {
        console.debug(e);
        return error.io;
      }
    }
  } else {
    console.debug(e);
    return error.io;
  }
}

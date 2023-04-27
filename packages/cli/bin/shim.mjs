import path from "node:path";
import fs from "node:fs";

// Shimmed implementation of `fd_readdir`.
// Workaround for a Node.js issue: https://github.com/nodejs/node/issues/47193

/**
 *
 * @param {any} original
 * @param {{ memory: WebAssembly.Memory }} memoryRef
 * @returns
 */
export function shim(original, memoryRef, rootDir) {
  const decoder = new TextDecoder();
  const fdCache = new Map(
    // 3 is a special root fd
    [
      [
        3,
        {
          path: rootDir,
        },
      ],
    ]
  );
  return {
    path_open: (
      fd,
      dirflags,
      path_ptr,
      path_len,
      oflags,
      fs_rights_base,
      fs_rights_inheriting,
      fdflags,
      bufused_ptr
    ) => {
      const parentFd = fdCache.get(fd);
      if (parentFd === undefined) {
        // badfd
        return 8;
      }
      const res = original.path_open(
        fd,
        dirflags,
        path_ptr,
        path_len,
        oflags,
        fs_rights_base,
        fs_rights_inheriting,
        fdflags,
        bufused_ptr
      );
      if (res === 0) {
        // success
        const dirPath = readString(path_ptr, path_len);
        const resultFd = new DataView(memoryRef.memory.buffer).getUint32(
          bufused_ptr,
          true
        );

        fdCache.set(resultFd, {
          path: path.join(parentFd.path, dirPath),
        });
      }
      return res;
    },
    fd_readdir: (fd, buf, buf_len, cookie, bufused_ptr) => {
      const fdData = fdCache.get(fd);
      if (fdData === undefined) {
        // badfd
        return 8;
      }
      cookie = Number(cookie);
      const allEntries =
        fdData.entries && cookie !== 0
          ? fdData.entries
          : fs.readdirSync(fdData.path, {
              withFileTypes: true,
              encoding: "buffer",
            });
      fdData.entries = allEntries;
      const entries = allEntries.slice(cookie);
      const bufView = new DataView(memoryRef.memory.buffer, buf, buf_len);
      let offset = 0;
      for (let index = 0; offset < buf_len && index < entries.length; index++) {
        if (offset + 24 >= buf_len) {
          // no space to write dirent
          offset = buf_len;
          break;
        }
        // write dirents until it buffer is full
        const entry = entries[index];
        // d_next
        bufView.setBigUint64(offset, BigInt(cookie + index + 1), true);
        // d_ino
        bufView.setBigUint64(offset + 8, BigInt(cookie + index), true);
        // d_namlen
        bufView.setUint32(offset + 16, entry.name.byteLength, true);
        // d_type
        bufView.setUint32(offset + 20, getFileType(entry), true);
        // name
        const uint8View = new Uint8Array(
          memoryRef.memory.buffer,
          buf + offset + 24,
          buf_len - offset - 24
        );
        uint8View.set(
          entry.name.subarray(0, Math.min(uint8View.length, entry.name.length))
        );

        offset += 24 + Math.min(uint8View.length, entry.name.length);
      }
      // Write offset
      new DataView(memoryRef.memory.buffer).setUint32(
        bufused_ptr,
        offset,
        true
      );
      return 0;
    },
    fd_close: (fd, bufused_ptr) => {
      const res = original.fd_close(fd, bufused_ptr);
      if (res === 0) {
        fdCache.delete(fd);
      }
      return res;
    },
  };

  /**
   * Reads string pointer.
   *
   * @param {ArrayBuffer} memory
   * @param {ptr}
   */
  function readString(ptr, len) {
    return decoder.decode(new Uint8Array(memoryRef.memory.buffer, ptr, len));
  }

  /**
   * Converts fs.DirEnt to file_type
   */
  function getFileType(dirent) {
    if (dirent.isDirectory()) {
      return 3; // directory
    }
    if (dirent.isFile()) {
      return 4; // regular_file
    }
    if (dirent.isBlockDevice()) {
      return 1; // block_device
    }
    if (dirent.isCharacterDevice()) {
      return 2; // character_device
    }
    if (dirent.isSymbolicLink()) {
      return 7; // symbolic_link
    }
    return 0; // unknown
  }
}

import type { BigIntStats, Dirent } from "node:fs";
import { DirEntry } from "./fs.js";

export function writePrestatDir(
  memory: ArrayBuffer,
  ptr: number,
  pr_name_len: number
) {
  const view = new DataView(memory);
  // preopentype::dir
  view.setUint32(ptr, 0, true);
  view.setUint32(ptr + 4, pr_name_len, true);
}

export function writeFilestat(
  memory: ArrayBuffer,
  ptr: number,
  dev: bigint,
  ino: bigint,
  filetype: number,
  nlink: bigint,
  size: bigint,
  atim: bigint,
  mtim: bigint,
  ctim: bigint
) {
  const view = new DataView(memory, ptr, 64);
  view.setBigUint64(0, dev, true);
  view.setBigUint64(8, ino, true);
  view.setUint8(16, filetype);
  view.setBigUint64(24, nlink, true);
  view.setBigUint64(32, size, true);
  view.setBigUint64(40, atim, true);
  view.setBigUint64(48, mtim, true);
  view.setBigUint64(56, ctim, true);
}

export function generateOneReaddirEntry(
  ent: DirEntry,
  index: number
): Uint8Array {
  const ret = new Uint8Array(24 + ent.name.length);
  const view = new DataView(ret.buffer);
  // d_next
  view.setBigUint64(0, BigInt(index) + 1n, true);
  // d_ino
  view.setBigUint64(8, BigInt(ent.ino), true);
  // d_namlen
  view.setUint32(16, ent.name.length, true);
  // d_type
  view.setUint8(20, getFiletypeOfStat(ent.rawEnt));
  // name
  ret.set(ent.name, 24);
  return ret;
}

export function getFiletypeOfStat(stat: BigIntStats | Dirent): number {
  if (stat.isDirectory()) {
    // directory
    return 3;
  } else if (stat.isFile()) {
    // regular_file
    return 4;
  } else if (stat.isSymbolicLink()) {
    // symbolic_link
    return 7;
  } else if (stat.isBlockDevice()) {
    // block_device
    return 1;
  } else if (stat.isCharacterDevice()) {
    // character_device
    return 2;
  } else {
    // unknown
    return 0;
  }
}

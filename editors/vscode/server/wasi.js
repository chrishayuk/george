// A minimal WASI (preview 1) host for Node: enough for the george program
// and its language server. They read standard input, write standard output
// and standard error, and george reads the program and data files it is
// given, which this host lets it read (and only read) from the real file
// system through one preopened directory, "/". It needs no other library.
"use strict";

const fs = require("fs");

const ERRNO_ACCES = 2;
const ERRNO_BADF = 8;
const ERRNO_INVAL = 28;
const ERRNO_ISDIR = 31;
const ERRNO_NOENT = 44;
const ERRNO_NOSYS = 52;
const ERRNO_NOTDIR = 54;
const FILETYPE_CHARACTER_DEVICE = 2;
const FILETYPE_DIRECTORY = 3;
const FILETYPE_REGULAR_FILE = 4;
const ALL_RIGHTS = 0xffffffffffffffffn;
const PREOPEN = 3; // the one directory, "/"

const pause = new Int32Array(new SharedArrayBuffer(4));
const sleep = (ms) => Atomics.wait(pause, 0, 0, ms);

// Standard input may be a pipe that is not ready yet: wait and try again.
function readInput(buffer) {
  for (;;) {
    try {
      return fs.readSync(0, buffer, 0, buffer.length, null);
    } catch (error) {
      if (error.code === "EOF") return 0;
      if (error.code !== "EAGAIN") throw error;
      sleep(5);
    }
  }
}

function writeOutput(fd, bytes) {
  let done = 0;
  while (done < bytes.length) {
    try {
      done += fs.writeSync(fd, bytes, done, bytes.length - done);
    } catch (error) {
      if (error.code !== "EAGAIN") throw error;
      sleep(5);
    }
  }
}

/**
 * Runs a WASI command to completion and returns its exit code.
 *
 * @param {string} wasmPath   the compiled program
 * @param {object} [options]
 * @param {string[]} [options.args]      arguments after the program name
 * @param {Uint8Array} [options.prefix]  bytes the program reads before
 *        anything from the real standard input
 */
function run(wasmPath, { args = [], prefix = new Uint8Array(0) } = {}) {
  const module = new WebAssembly.Module(fs.readFileSync(wasmPath));
  let memory;
  let position = 0;
  const view = () => new DataView(memory.buffer);
  const bytes = () => new Uint8Array(memory.buffer);
  const argv = ["george", ...args].map((arg) => Buffer.from(arg + "\0"));
  /** @type {Map<number, {data: Uint8Array, at: number}>} */
  const open = new Map();
  let nextFd = PREOPEN + 1;
  const typeOf = (fd) =>
    fd <= 2 ? FILETYPE_CHARACTER_DEVICE : fd === PREOPEN ? FILETYPE_DIRECTORY : open.has(fd) ? FILETYPE_REGULAR_FILE : 0;

  const wasi = {
    args_sizes_get(countPtr, sizePtr) {
      view().setUint32(countPtr, argv.length, true);
      view().setUint32(sizePtr, argv.reduce((n, a) => n + a.length, 0), true);
      return 0;
    },
    args_get(argvPtr, bufPtr) {
      for (const arg of argv) {
        view().setUint32(argvPtr, bufPtr, true);
        bytes().set(arg, bufPtr);
        argvPtr += 4;
        bufPtr += arg.length;
      }
      return 0;
    },
    environ_sizes_get(countPtr, sizePtr) {
      view().setUint32(countPtr, 0, true);
      view().setUint32(sizePtr, 0, true);
      return 0;
    },
    environ_get() {
      return 0;
    },
    random_get(ptr, length) {
      require("crypto").randomFillSync(bytes(), ptr, length);
      return 0;
    },
    fd_read(fd, iovsPtr, iovsLen, nreadPtr) {
      if (open.has(fd)) {
        const file = open.get(fd);
        let total = 0;
        for (let i = 0; i < iovsLen; i++) {
          const ptr = view().getUint32(iovsPtr + i * 8, true);
          const len = view().getUint32(iovsPtr + i * 8 + 4, true);
          const chunk = file.data.subarray(file.at, file.at + len);
          bytes().set(chunk, ptr);
          file.at += chunk.length;
          total += chunk.length;
          if (chunk.length < len) break;
        }
        view().setUint32(nreadPtr, total, true);
        return 0;
      }
      if (fd !== 0) return ERRNO_BADF;
      let total = 0;
      for (let i = 0; i < iovsLen && total === 0; i++) {
        const ptr = view().getUint32(iovsPtr + i * 8, true);
        const len = view().getUint32(iovsPtr + i * 8 + 4, true);
        if (len === 0) continue;
        if (position < prefix.length) {
          const chunk = prefix.subarray(position, position + len);
          bytes().set(chunk, ptr);
          position += chunk.length;
          total = chunk.length;
        } else {
          const buffer = Buffer.alloc(len);
          total = readInput(buffer);
          bytes().set(buffer.subarray(0, total), ptr);
          if (total === 0) break;
        }
      }
      view().setUint32(nreadPtr, total, true);
      return 0;
    },
    fd_write(fd, iovsPtr, iovsLen, nwrittenPtr) {
      if (fd !== 1 && fd !== 2) return ERRNO_BADF;
      let total = 0;
      for (let i = 0; i < iovsLen; i++) {
        const ptr = view().getUint32(iovsPtr + i * 8, true);
        const len = view().getUint32(iovsPtr + i * 8 + 4, true);
        if (len > 0) writeOutput(fd, bytes().slice(ptr, ptr + len));
        total += len;
      }
      view().setUint32(nwrittenPtr, total, true);
      return 0;
    },
    fd_fdstat_get(fd, statPtr) {
      const type = typeOf(fd);
      if (!type) return ERRNO_BADF;
      bytes().fill(0, statPtr, statPtr + 24);
      view().setUint8(statPtr, type);
      view().setBigUint64(statPtr + 8, ALL_RIGHTS, true);
      view().setBigUint64(statPtr + 16, ALL_RIGHTS, true);
      return 0;
    },
    fd_filestat_get(fd, statPtr) {
      const type = typeOf(fd);
      if (!type) return ERRNO_BADF;
      bytes().fill(0, statPtr, statPtr + 64);
      view().setUint8(statPtr + 16, type);
      view().setBigUint64(statPtr + 24, 1n, true);
      view().setBigUint64(statPtr + 32, BigInt(open.get(fd)?.data.length ?? 0), true);
      return 0;
    },
    fd_seek(fd, offset, whence, newOffsetPtr) {
      const file = open.get(fd);
      if (!file) return ERRNO_BADF;
      const base = [0, file.at, file.data.length][whence];
      const at = base + Number(offset);
      if (base === undefined || at < 0) return ERRNO_INVAL;
      file.at = at;
      view().setBigUint64(newOffsetPtr, BigInt(at), true);
      return 0;
    },
    fd_close(fd) {
      open.delete(fd);
      return 0;
    },
    clock_time_get(id, precision, timePtr) {
      view().setBigUint64(timePtr, BigInt(Date.now()) * 1000000n, true);
      return 0;
    },
    // One preopened directory, "/": files may be opened to be read.
    fd_prestat_get(fd, ptr) {
      if (fd !== PREOPEN) return ERRNO_BADF;
      view().setUint8(ptr, 0);
      view().setUint32(ptr + 4, 1, true);
      return 0;
    },
    fd_prestat_dir_name(fd, ptr, len) {
      if (fd !== PREOPEN) return ERRNO_BADF;
      if (len >= 1) bytes()[ptr] = 47; // "/"
      return 0;
    },
    path_open(dirFd, dirFlags, pathPtr, pathLen, oflags, rightsBase, rightsInheriting, fdFlags, fdPtr) {
      if (dirFd !== PREOPEN) return ERRNO_NOTDIR;
      if (oflags & 0b1101) return ERRNO_ACCES; // create, exclusive or truncate: nothing is written
      let path = "/" + Buffer.from(bytes().slice(pathPtr, pathPtr + pathLen)).toString().replace(/^\.?\/+/, "");
      // On Windows, "C:/..." arrives as a path under "/".
      if (/^\/[A-Za-z]:\//.test(path)) path = path.slice(1);
      let data;
      try {
        if (fs.statSync(path).isDirectory()) return ERRNO_ISDIR;
        data = fs.readFileSync(path);
      } catch (error) {
        return error.code === "EACCES" ? ERRNO_ACCES : ERRNO_NOENT;
      }
      const fd = nextFd++;
      open.set(fd, { data, at: 0 });
      view().setUint32(fdPtr, fd, true);
      return 0;
    },
    proc_exit(code) {
      throw new Exit(code);
    },
  };

  // Which calls a build imports depends on the toolchain that made it. One
  // this host has no answer for is given a stub that says "not supported".
  for (const { module: from, name } of WebAssembly.Module.imports(module)) {
    if (from === "wasi_snapshot_preview1" && !(name in wasi)) wasi[name] = () => ERRNO_NOSYS;
  }
  const instance = new WebAssembly.Instance(module, { wasi_snapshot_preview1: wasi });
  memory = instance.exports.memory;
  try {
    instance.exports._start();
    return 0;
  } catch (error) {
    if (error instanceof Exit) return error.code;
    throw error;
  }
}

class Exit extends Error {
  constructor(code) {
    super(`exit ${code}`);
    this.code = code;
  }
}

module.exports = { run };

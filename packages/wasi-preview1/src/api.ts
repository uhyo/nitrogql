/**
 * https://github.com/WebAssembly/WASI/blob/main/legacy/preview1/docs.md
 */
export type WASIAPI = {
  args_sizes_get: (args_count_buf: number, args_size_buf: number) => number;
  args_get: (argv: number, argv_buf: number) => number;
  environ_sizes_get: (
    environ_count_buf: number,
    environ_size_buf: number
  ) => number;
  environ_get: (environ: number, environ_buf: number) => number;
  fd_prestat_get: (fd: number, buf: number) => number;
  fd_prestat_dir_name: (fd: number, path: number, path_len: number) => number;
  path_open: (
    fd: number,
    dirflags: number,
    path_ptr: number,
    path_len: number,
    oflags: number,
    fs_rights_base: number,
    fs_rights_inheriting: number,
    fdflags: number,
    ret_buf: number
  ) => number;
  fd_close: (fd: number) => number;
  fd_filestat_get: (fd: number, ret_buf: number) => number;
  fd_read: (
    fd: number,
    iovs_ptr: number,
    iovs_len: number,
    ret_buf: number
  ) => number;
  fd_write: (
    fd: number,
    iovs_ptr: number,
    iovs_len: number,
    ret_buf: number
  ) => number;
  fd_readdir: (
    fd: number,
    buf: number,
    buf_len: number,
    cookie: bigint,
    ret_buf: number
  ) => number;
  path_filestat_get: (
    fd: number,
    flags: number,
    path: number,
    path_len: number,
    ret_buf: number
  ) => number;
  path_create_directory: (fd: number, path: number, path_len: number) => number;
  random_get: (buf: number, buf_len: number) => number;
  clock_time_get: (id: number, precision: number, ret_buf: number) => number;
  proc_exit: (rval: number) => void;
};

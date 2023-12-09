/**
 * API exposed by crates/graphql-loader.
 */
export type WASMInterface = {
  memory: WebAssembly.Memory;
  init(debug: number): void;
  alloc_string(size: number): number;
  free_string(ptr: number, size: number): void;
  get_result_ptr(): number;
  get_result_size(): number;
  load_config(input_ptr: number, input_size: number): number;
  initiate_task(
    filename_ptr: number,
    filename_size: number,
    input_ptr: number,
    input_size: number
  ): number;
  get_required_files(task_id: number): number;
  load_file(
    task_id: number,
    filename_ptr: number,
    filename_size: number,
    input_ptr: number,
    input_size: number
  ): number;
  emit_js(task_id: number): number;
  free_task(task_id: number): void;
  get_log(): void;
};

export type WASMInstance = WebAssembly.Instance & {
  exports: WASMInterface;
};

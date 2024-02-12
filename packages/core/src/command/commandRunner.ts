import { TransformStream } from "node:stream/web";

/**
 * A command runner is responsible for maintaining a queue of commands and
 * executing them in order.
 * A command is a JavaScript program that runs as a module.
 */
export class CommandRunner {
  #stream = new CommandTransform();
  #writer = this.#stream.writable.getWriter();
  output = this.#stream.readable;

  /**
   * Enqueues a command to be executed.
   */
  run(program: string): void {
    this.#writer.write(program);
  }
}

class CommandTransform extends TransformStream<string, CommandResult> {
  constructor() {
    super({
      transform(chunk, controller) {
        import(`data:text/javascript,${encodeURIComponent(chunk)}`)
          .then((module) => {
            controller.enqueue({
              result: module.default,
            });
          })
          .catch((err) => {
            controller.enqueue({
              result: null,
              error: err,
            });
          });
      },
    });
  }
}

export type CommandResult = {
  /**
   * The result of the command.
   */
  result: unknown;
  /**
   * The error thrown by the command.
   */
  error?: unknown;
};

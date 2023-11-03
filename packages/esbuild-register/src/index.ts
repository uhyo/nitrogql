import { register } from "node:module";

const hookUrl = new URL("hook.js", import.meta.url);
register(hookUrl);

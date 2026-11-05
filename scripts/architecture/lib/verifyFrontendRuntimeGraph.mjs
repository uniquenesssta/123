import assert from "node:assert/strict";
import ts from "typescript";
import { frontendRuntimeGraph, runtimeGraphViolations } from "./frontendRuntimeGraph.mjs";

/** Isolated inputs exercise the original module gate's runtime-cycle policy. */
export function verifyFrontendRuntimeGraph() {
  const options = { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ESNext, isolatedModules: true };
  const scan = sources => frontendRuntimeGraph(new Map(Object.entries(sources)), options, (_file, specifier) => `${specifier.replace(/^\.\//, "")}.ts`);
  let count = 0;
  const check = run => { run(); count++; };
  check(() => assert.equal(runtimeGraphViolations(scan({ "a.ts": "import {b} from './b'; export const a=()=>b;", "b.ts": "export const b=1;" })).length, 0));
  check(() => assert.match(runtimeGraphViolations(scan({ "a.ts": "export {b} from './b';", "b.ts": "export {a} from './a';" }))[0], /运行时依赖环/));
  check(() => assert.equal(scan({ "a.ts": "import type {B} from './b'; export interface A {b:B}", "b.ts": "import type {A} from './a'; export interface B {a:A}" }).cycles.length, 0));
  check(() => assert.equal(scan({ "a.ts": "import {B} from './b'; export const a:B={};", "b.ts": "export interface B {}" }).graph.get("a.ts").length, 0));
  check(() => assert.equal(scan({ "a.ts": "import './b';", "b.ts": "import './a';" }).cycles.length, 1));
  check(() => assert.equal(scan({ "a.ts": "export * from './a';" }).cycles.length, 1));
  check(() => { const result=scan({ "a.ts": "export const load=()=>import('./b');", "b.ts": "import './a';" }); assert.equal(result.cycles.length,0); assert.equal(result.dynamic[0].to,"b.ts"); });
  check(() => assert.match(runtimeGraphViolations(scan({ "a.ts": "export const load=(path:string)=>import(path);" }))[0], /未解析的模块加载/));
  check(() => assert.match(runtimeGraphViolations(scan({ "a.ts": "export const load=(path:string)=>require(path);" }))[0], /未解析的模块加载/));
  check(() => assert.equal(scan({ "a.ts": "require('./b');", "b.ts": "require('./a');" }).cycles.length, 1));
  check(() => assert.equal(scan({ "a.ts": "const x='import(unknown)'; // import('./a')\nexport {x};" }).edges.length, 0));
  check(() => assert.equal(scan({ "a.ts": "export const load=()=>import(`./b`);", "b.ts": "export const b=1;" }).dynamic.length, 1));
  check(() => assert.ok(runtimeGraphViolations(scan({ "a.ts": "export const broken = ;" })).length > 0));
  check(() => assert.equal(scan({ "a.d.ts": "import('./a');" }).edges.length, 0));
  console.log(`R16-09 runtime graph: ${count} value/type/cycle/dynamic checks PASS`);
  return count;
}

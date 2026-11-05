import ts from "typescript";
import { findCycles } from "./cargo.mjs";

/** Follow emitted value dependencies; erased TypeScript references are not runtime cycles. */
export function frontendRuntimeGraph(sources, compilerOptions, resolveImport) {
  const graph = new Map([...sources.keys()].map(file => [file, []]));
  const edges = [], dynamic = [], unknown = [], errors = [];
  for (const [file, source] of sources) {
    if (file.endsWith(".d.ts")) continue;
    const parsed = ts.createSourceFile(file, source, ts.ScriptTarget.Latest, true);
    for (const diagnostic of parsed.parseDiagnostics) errors.push(`${file}: ${ts.flattenDiagnosticMessageText(diagnostic.messageText, " ")}`);
    const emitted = ts.transpileModule(source, { fileName: file, compilerOptions }).outputText;
    const ast = ts.createSourceFile(`${file}.js`, emitted, ts.ScriptTarget.Latest, true, ts.ScriptKind.JS);
    function add(specifier, kind) {
      const target = resolveImport(file, specifier);
      const edge = { from: file, to: target, specifier, kind };
      edges.push(edge);
      if (kind === "dynamic") dynamic.push(edge);
      else if (graph.has(target)) graph.get(file).push(target);
    }
    function visit(node) {
      if ((ts.isImportDeclaration(node) || ts.isExportDeclaration(node)) && node.moduleSpecifier && ts.isStringLiteralLike(node.moduleSpecifier)) {
        add(node.moduleSpecifier.text, "static");
      } else if (ts.isCallExpression(node) && (node.expression.kind === ts.SyntaxKind.ImportKeyword || (ts.isIdentifier(node.expression) && node.expression.text === "require"))) {
        const kind = node.expression.kind === ts.SyntaxKind.ImportKeyword ? "dynamic" : "require";
        const argument = node.arguments[0];
        if (argument && ts.isStringLiteralLike(argument)) add(argument.text, kind);
        else unknown.push({ file, kind, expression: node.getText(ast) });
      }
      ts.forEachChild(node, visit);
    }
    visit(ast);
  }
  return { graph, edges, dynamic, unknown, errors, cycles: findCycles(graph) };
}

export function runtimeGraphViolations(result) {
  return [
    ...result.errors,
    ...result.cycles.map(cycle => `前端运行时依赖环：${cycle.join(" -> ")}`),
    ...result.unknown.map(item => `未解析的模块加载，需明确实际消费者：${item.file} ${item.expression}`),
  ];
}

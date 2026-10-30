import {readFileSync,readdirSync} from "node:fs";
import {resolve,dirname,relative,join} from "node:path";
import postcss from "postcss";
import ts from "typescript";
export function sourceFiles(directory,extension){return readdirSync(directory,{withFileTypes:true}).flatMap(entry=>entry.isDirectory()?sourceFiles(join(directory,entry.name),extension):entry.name.endsWith(extension)?[join(directory,entry.name)]:[]);}
/** Inspect the actual CSS graph; never trust a filename list as proof of runtime loading. */
export function inspectStyleGraph(root,entry="src/styles/index.css",read=(file)=>readFileSync(file,"utf8")){
 const files=[],sheets=new Map(),seen=new Set();
 function visit(file,ancestors=[]){const full=resolve(root,file),path=relative(root,full).replaceAll("\\","/");if(!path.startsWith("src/")||path.includes("../")||!path.endsWith(".css"))throw Error("Stylesheet outside source boundary: "+file);if(ancestors.includes(full))throw Error("Cyclic stylesheet: "+path);if(seen.has(full))throw Error("Duplicate stylesheet load: "+path);seen.add(full);files.push(path);const ast=postcss.parse(read(full),{from:full});sheets.set(path,ast);let rulesStarted=false;for(const node of ast.nodes){if(node.type==="comment")continue;if(node.type==="atrule"&&node.name==="import"){if(rulesStarted)throw Error("Late CSS import: "+path);const match=node.params.match(/^["']([^"']+)["']$/);if(!match||/^(?:[a-z]+:|\/|#)/i.test(match[1]))throw Error("Uncontrolled CSS import: "+path);visit(resolve(dirname(full),match[1]),[...ancestors,full]);}else rulesStarted=true;}}
 visit(entry);return {files,sheets};
}
export function productionStyleFiles(root){return inspectStyleGraph(root).files;}
export function findCssImports(file,text){
 const source=ts.createSourceFile(file,text,ts.ScriptTarget.Latest,true),found=[];
 const css=specifier=>/\.css(?:[?#]|$)/.test(specifier);
 function visit(node){
  if((ts.isImportDeclaration(node)||ts.isExportDeclaration(node))&&node.moduleSpecifier&&ts.isStringLiteral(node.moduleSpecifier)&&css(node.moduleSpecifier.text))found.push({file,specifier:node.moduleSpecifier.text,...(ts.isExportDeclaration(node)?{kind:"re-export"}:node.importClause?{kind:"named-import"}:{})});
  if(ts.isCallExpression(node)&&node.expression.kind===ts.SyntaxKind.ImportKeyword&&node.arguments.length&&ts.isStringLiteral(node.arguments[0])&&css(node.arguments[0].text))found.push({file,specifier:node.arguments[0].text,kind:"dynamic-import"});
  ts.forEachChild(node,visit);
 }visit(source);return found;
}
export function runtimeCssImports(root){return [".ts",".tsx",".js",".jsx"].flatMap(extension=>sourceFiles(resolve(root,"src"),extension)).filter(p=>!p.includes("__tests__")).flatMap(file=>findCssImports(relative(root,file).replaceAll("\\","/"),readFileSync(file,"utf8")));}
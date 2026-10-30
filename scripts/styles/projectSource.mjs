import {readFileSync} from "node:fs";
import {resolve,relative} from "node:path";
import {fileURLToPath} from "node:url";
import {readStylesheet} from "../tokens/tokenStyles.mjs";
/** Existing source contracts inspect resolved CSS declarations after module decomposition. Binary reads stay raw. */
export function readProjectSource(root,path,options){
 const encoding=typeof options==="string"?options:options?.encoding;
 const file=path instanceof URL?fileURLToPath(path):path;
 if(typeof file==="string"&&file.endsWith(".css")&&["utf8","utf-8"].includes(encoding))return readStylesheet(root,relative(root,resolve(root,file)));
 return readFileSync(path,options);
}

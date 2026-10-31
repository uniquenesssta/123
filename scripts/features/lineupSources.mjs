import {readFileSync,readdirSync} from "node:fs";import {join} from "node:path";
/** Scan the actual migrated workflow owners together with their composition root. */
export function readLineupWorkflowSources(root) { return ["src/main.ts",...readdirSync(join(root,"src/features/lineups")).filter(file=>file.endsWith(".ts")).map(file=>"src/features/lineups/"+file)].map(file=>readFileSync(join(root,file),"utf8")).join("\n"); }

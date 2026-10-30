import { assetManifest } from "./assetManifest";
export interface ControlledAsset { readonly path: string; readonly kind: "vector"|"font"|"license"; readonly bytes:number; readonly sha256:string; readonly width?:number; readonly height?:number; }
/** The Vite source graph owns loading. Registry metadata never opens files or fetches URLs. */
export function controlledAsset(id: string): ControlledAsset {
  if(!Object.hasOwn(assetManifest,id))throw new Error("Unregistered UI asset: "+id);
  const asset=assetManifest[id];
  if(!asset.path.startsWith("src/assets/")||asset.path.includes("..")||asset.path.includes(":")||asset.bytes<=0)throw new Error("Invalid controlled UI asset: "+id);
  return asset;
}

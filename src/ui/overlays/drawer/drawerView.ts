import { dialogView, type DialogViewOptions } from "../dialog";
export function drawerView(o: DialogViewOptions): string { return dialogView({...o,width:480,className:`ui-drawer ${o.className??""}`}); }

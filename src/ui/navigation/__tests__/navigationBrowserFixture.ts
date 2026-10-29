import { tabsView, segmentedControlView, paginationView, updatePaginationView, NavigationKeyboardController, workspaceSectionNavigation, workspaceAnchorNavigation } from "../index";
export function installNavigationFixture() {
    const root = document.querySelector<HTMLElement>("#navigation-root")!, checks: string[] = [];
    const q = <T extends HTMLElement>(s: string) => root.querySelector<T>(s)!;
    const expect = (condition: unknown, message: string) => { if (!condition)
        throw new Error(message); checks.push(message); };
    let active = "one", mode = "detail", page = 1, busy = false, activations = 0, closes = 0;
    const controller = new NavigationKeyboardController(root);
    controller.start();
    controller.start();
    const items = [{ id: "one", label: "首尔 FC", panelId: "object-one", closable: true, attributes: { "data-object": "one" }, closeAttributes: { "data-close": "one" } }, { id: "disabled", label: "暂不可用", panelId: "object-disabled", disabled: true, closable: true }, { id: "two", label: "这是一个非常长的球队对象名称需要保留关闭按钮并且单行省略", panelId: "object-two", closable: true, attributes: { "data-object": "two" }, closeAttributes: { "data-close": "two" } }];
    function paint() {
        root.innerHTML = `<h1>导航与布局组件验收</h1><p>测试夹具 · 不读取数据库</p><section><h2>对象标签 · 32</h2>${tabsView({ id: "objects", label: "打开的球队", items, activeId: active })}<div id="object-one" role="tabpanel" aria-labelledby="objects-one" ${active !== "one" ? "hidden" : ""}>首尔 FC 档案</div><div id="object-disabled" role="tabpanel" aria-labelledby="objects-disabled" hidden></div><div id="object-two" role="tabpanel" aria-labelledby="objects-two" ${active !== "two" ? "hidden" : ""}>另一球队档案</div></section><section><h2>对象标签 · 40</h2>${tabsView({ id: "large", label: "大号对象标签", items: [{ id: "large", label: "球员档案", panelId: "large-panel", closable: true }], activeId: "large", size: 40 })}<div id="large-panel" role="tabpanel" aria-labelledby="large-large">球员资料</div></section><section><h2>显示模式</h2>${segmentedControlView({ label: "显示模式", items: [{ value: "detail", label: "详情", icon: "panel-left", attributes: { "data-mode": "detail" } }, { value: "compare", label: "比较", icon: "users", attributes: { "data-mode": "compare" } }, { value: "cards", label: "概览", icon: "dashboard", disabled: true }], value: mode })}${segmentedControlView({ label: "大号显示模式", size: 40, items: [{ value: "detail", label: "详情" }, { value: "compare", label: "比较", disabled: true }], value: "detail" })}</section><section><h2>结果分页</h2>${paginationView({ page, hasMore: page < 3, busy, previousAction: "previous", nextAction: "next" })}</section><section><h2>工作区入口</h2>${workspaceSectionNavigation([{ id: "directory", index: "01", label: "球队目录" }, { id: "blocked", index: "02", label: "完整档案", disabled: true }, { id: "create", index: "03", label: "新增资料" }], "directory")}</section><section><h2>竖排档案目录</h2>${workspaceAnchorNavigation("档案目录",[{id:"anchor-one",label:"基础身份"},{id:"anchor-two",label:"教练任期"}],"vertical")}</section>`;
    }
    root.addEventListener("click", event => {
        const b = (event.target as Element).closest<HTMLButtonElement>("button");
        if (!b || b.disabled)
            return;
        if (b.dataset.object) {
            active = b.dataset.object;
            activations++;
            paint();
            q<HTMLButtonElement>("[data-object=" + active + "]").focus();
        }
        else if (b.hasAttribute("data-tab-close")) {
            closes++;
        }
        else if (b.dataset.mode) {
            mode = b.dataset.mode;
            paint();
            q<HTMLButtonElement>("[data-mode=" + mode + "]").focus();
        }
        else if (b.dataset.action === "next" || b.dataset.action === "previous") {
            page += b.dataset.action === "next" ? 1 : -1;
            busy = true;
            updatePaginationView(q(".ui-pagination"),{page,hasMore:page<3,busy});
        }
    });
    paint();
    function structure() {
        expect(q(".ui-tabs").getBoundingClientRect().height === 32, "Tabs32 exact height");
        expect(q("#large").getBoundingClientRect().height === 40, "Tabs40 exact height");
        const tabs = Array.from(root.querySelectorAll<HTMLElement>(".ui-tab"));
        expect(tabs.every(t => t.getBoundingClientRect().width <= 200.1), "tab maximum width and ellipsis protect close target");
        expect(Array.from(root.querySelectorAll<HTMLElement>(".ui-tab-close")).every(t => t.offsetWidth === 24 && t.offsetHeight === 24), "close hit target24 independent of label");
        expect(root.querySelectorAll('#objects [role="tab"][tabindex="0"]').length === 1, "one object tab in sequential tab order");
        expect(Array.from(root.querySelectorAll<HTMLElement>('[role="tab"]')).every(t => !!document.getElementById(t.getAttribute("aria-controls")!)), "all tab panels resolve");
        expect(q(".ui-segmented").offsetHeight === 32 && q('.ui-segmented[data-fip-control-scale="40"]').offsetHeight === 40, "segmented32/40 exact outer height");
        expect(q(".ui-segmented button").offsetHeight === 28 && q('.ui-segmented[data-fip-control-scale="40"] button').offsetHeight === 34, "segmented inner height accounts for padding");
        expect(q<HTMLButtonElement>('[data-action="previous"]').disabled, "first page prevents previous");
        expect(!q<HTMLButtonElement>('[data-action="next"]').disabled, "has_more controls next");
        expect(q(".ui-pagination").offsetHeight >= 48, "pagination bounded footer alignment");
        expect(q('[data-action="next"]').offsetHeight === 27, "pagination27px controls");
        expect(document.documentElement.scrollWidth <= innerWidth, "no horizontal document overflow");
        expect(getComputedStyle(q(".ui-tab [role=tab]")).fontWeight === "500", "label500 weight");
        const row = q(".ui-pagination"), buttons = Array.from(row.querySelectorAll<HTMLElement>("button"));
        expect(buttons.every(b => Math.abs(b.getBoundingClientRect().top + b.offsetHeight / 2 - (row.getBoundingClientRect().top + row.offsetHeight / 2)) < 1), "pagination row controls share vertical center");
        const sections=Array.from(root.querySelectorAll<HTMLElement>(".workspace-section-nav-item")); expect(sections.every(b=>b.offsetHeight===32),"standalone section controls keep32px height"); expect(sections.every(b=>Math.abs(b.getBoundingClientRect().top-sections[0].getBoundingClientRect().top)<1),"standalone section controls align horizontally"); expect(sections.every(b=>b.querySelector<HTMLElement>("strong")!.scrollWidth<=b.querySelector<HTMLElement>("strong")!.clientWidth),"section labels are not truncated by legacy grid columns"); for(const activeId of [null,"missing","disabled"]){const host=document.createElement("div");host.innerHTML=tabsView({id:"unselected",label:"未选中对象",items,activeId});expect(!host.querySelector('[aria-selected="true"]'),"null/missing/disabled identity does not invent selected data: "+activeId);expect(host.querySelectorAll('[role="tab"][tabindex="0"]').length===1,"unselected state keeps first enabled focus stop");} const escaped=tabsView({ id: "escape", label: "<unsafe>", items: [{ id: "safe", label: '<img src=x onerror=alert(1)>', panelId: "safe-panel" }], activeId: "safe" });
        expect(!escaped.includes("<img"), "object names escaped");
        const pagination = q(".ui-pagination"), next = q<HTMLButtonElement>('[data-action="next"]');
        next.focus();
        updatePaginationView(pagination, { page: 2, hasMore: true, busy: false });
        expect(document.activeElement === next && q('[data-action="next"]') === next, "in-place update preserves focused button identity");
        expect(q("[data-pagination-current]").textContent === "2", "local table refresh updates current page");
        updatePaginationView(pagination, { page: 2, hasMore: true, busy: true });
        expect(next.disabled && pagination.getAttribute("aria-busy") === "true" && !q("[data-pagination-status]").hidden, "real local loading adapter disables and announces");
        expect(document.activeElement===q("[data-pagination-current]"),"busy parks focus on current page before disabling the trigger");
        updatePaginationView(pagination, { page: 1, hasMore: true, busy: false });
        expect(!next.disabled && q<HTMLButtonElement>('[data-action="previous"]').disabled, "local completion derives boundaries without a shadow disabled-state cache");
        expect(document.activeElement===q("[data-pagination-current]"),"completion keeps focus in pagination without stealing focus elsewhere");
        for (const invalid of [0, -1, NaN, 1.5]) {
            let threw = false;
            try {
                paginationView({ page: invalid, hasMore: false, previousAction: "p", nextAction: "n" });
            }
            catch {
                threw = true;
            }
            expect(threw, "reject invalid pagination " + invalid);
        }
    }
    function assertFocus(target:HTMLElement,background:HTMLElement=target){
      expect(target===document.activeElement&&target.matches(":focus-visible"),"native keyboard focus-visible is active");
      const luminance=(color:string)=>{const rgb=(color.match(/[\d.]+/g)??[]).slice(0,3).map(Number).map(v=>{v/=255;return v<=.04045?v/12.92:((v+.055)/1.055)**2.4;});return .2126*rgb[0]+.7152*rgb[1]+.0722*rgb[2];};
      const outline=luminance(getComputedStyle(background).outlineColor),surface=luminance(getComputedStyle(background).backgroundColor);
      expect((Math.max(outline,surface)+.05)/(Math.min(outline,surface)+.05)>=3,"selected keyboard focus ring contrast >=3:1 "+JSON.stringify({outline:getComputedStyle(background).outlineColor,surface:getComputedStyle(background).backgroundColor,outlineLuminance:outline,surfaceLuminance:surface,selector:background.matches(":has(button:focus-visible)"),active:background.className,rule:getComputedStyle(background).outlineStyle,width:getComputedStyle(background).outlineWidth}));
    }
    return { checks, q, expect, structure, assertFocus, controller, get active() { return active; }, get mode() { return mode; }, get page() { return page; }, get activations() { return activations; }, get closes() { return closes; }, settle() { busy = false; updatePaginationView(q(".ui-pagination"),{page,hasMore:page<3,busy}); }, focusFirst() { q<HTMLButtonElement>('[data-object="one"]').focus(); }, reset() { active = "one"; page = 1; busy = false; paint(); } };
}

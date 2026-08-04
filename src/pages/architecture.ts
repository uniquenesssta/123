import { taskContextRibbon, taskPageHeader } from "../components/taskWorkspace";
import { workspaceTaskAnchorNavigation } from "../components/workspace";

export function architecturePage(): string {
  const navigation = workspaceTaskAnchorNavigation([
    { id: "architecture-flow", index: "01", label: "系统边界", description: "界面、业务、模型与数据" },
    { id: "architecture-principles", index: "02", label: "运行原则", description: "历史与自动路由" },
    { id: "architecture-modules", index: "03", label: "底层模块", description: "开发与维护边界" },
  ]);
  return `<section class="module-workspace-page management-module-workspace">
    ${taskPageHeader({ eyebrow: "系统信息", title: "平台如何保证长期可维护", description: "日常使用不需要理解底层代码；这里用业务语言说明各层责任、历史原则与自动路由边界。", status: { label: "只读说明", tone: "neutral" } })}
    ${taskContextRibbon([
      { label: "界面层", value: "输入与结果展示", note: "不直接执行模型和数据库逻辑", tone: "neutral" },
      { label: "业务层", value: "流程与责任边界", note: "串联赛事、阵容、推演和复盘", tone: "accent" },
      { label: "模型层", value: "统一接口运行", note: "新增模型不破坏其他模块", tone: "accent" },
      { label: "数据层", value: "历史可追溯", note: "正式记录不覆盖", tone: "success" },
    ])}
    <div class="core-local-navigation">${navigation}</div>
    <div class="management-module-stage" data-workspace-scroll-key="architecture-stage">
      <section id="architecture-flow" class="management-section workspace-anchor-target"><div class="architecture-flow user-architecture"><article><span>1</span><h2>界面</h2><p>只负责输入、查看结果和管理数据，不直接执行模型和数据库逻辑。</p></article><article><span>2</span><h2>业务流程</h2><p>负责赛事选择、模型路由、球员目录、阵容和推演流程。</p></article><article><span>3</span><h2>模型</h2><p>通过统一接口运行，未来可以增加新的赛事模型而不破坏其他功能。</p></article><article><span>4</span><h2>数据</h2><p>数据服务保存赛事、球员、能力、阵容、运行结果和审计记录。</p></article></div></section>
      <section id="architecture-principles" class="two-column management-section workspace-anchor-target"><article class="panel"><div class="panel-heading"><div><span>数据原则</span><h2>历史不覆盖</h2></div></div><div class="domain-list user-domain-list"><div><b>能力</b><span>每次记录都保留历史，页面只显示当前有效值</span></div><div><b>阵容</b><span>每次修改形成新修订，旧版本仍可追溯</span></div><div><b>推演</b><span>保存本次使用的规则、模型、参数和输入数据版本</span></div></div></article><article class="panel"><div class="panel-heading"><div><span>自动选择</span><h2>赛事自动匹配</h2></div></div><div class="domain-list user-domain-list"><div><b>优先</b><span>本场指定规则或赛事阶段规则</span></div><div><b>其次</b><span>赛季与具体赛事绑定</span></div><div><b>默认</b><span>赛事类型对应的默认规则</span></div></div></article></section>
      <details id="architecture-modules" class="panel disclosure-panel management-section workspace-anchor-target"><summary><div><span>开发信息</span><strong>查看底层模块名称</strong></div><b>展开</b></summary><div class="disclosure-content domain-list user-domain-list"><div><b>桌面客户端层</b><span>负责页面显示和操作响应</span></div><div><b>业务流程层</b><span>串联录入、推演、复盘和分析</span></div><div><b>数据规则层</b><span>统一管理赛事、球队、球员和比赛数据</span></div><div><b>模型计算层</b><span>负责赛事计算和结果解释</span></div><div><b>数据库访问层</b><span>负责保存数据和自动升级数据库结构</span></div></div></details>
    </div>
  </section>`;
}

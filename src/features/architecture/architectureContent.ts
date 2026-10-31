export function architecturePrinciples(): string {
  return `<div class="architecture-principles"><article class="panel"><h2>正常业务修订保留历史</h2><div class="domain-list user-domain-list"><div><b>能力</b><span>追加观察与生效记录；当前有效值不是删除旧值。</span></div><div><b>阵容</b><span>每次变更形成修订，保留旧版本与来源。</span></div><div><b>推演</b><span>保存本次规则、模型、参数及输入版本引用。</span></div></div></article><article class="panel"><h2>按上下文匹配有效规则</h2><p>明确指定规则 → 阶段 → 赛季 → 赛事 → 类型默认。</p><p>自动绑定还需满足启用状态、时间有效期、赛事类型与模型筛选；没有匹配时明确报错，不凭空补默认。</p></article></div>`;
}
export function architectureOverview(): string {
  const layers = [
    ["输入与结果展示", "界面", "收集输入、显示结果和响应操作；通过应用边界调用业务，不把数据库或模型计算直接放进页面。"],
    ["流程编排与规则校验", "业务流程", "串联赛事、球员、阵容、推演与复盘；核对上下文、输入条件及操作权限。"],
    ["契约与计算边界", "模型接口", "通过统一接口接入外部计算实现；接口或规则已登记，不等于本机已具备可运行模型。"],
    ["版本、结果与审计", "数据访问", "保存正式数据与引用关系，负责查询、持久化和结构迁移；保留历史不是自动备份。"],
  ];
  return `<div class="architecture-flow user-architecture">${layers.map(([note,title,body]) => `<article class="panel"><p>${note}</p><h2>${title}</h2><p>${body}</p></article>`).join("")}</div><p class="architecture-flow-note">职责示意：界面调用业务流程，业务协调模型接口与数据访问；并非正在执行的任务进度。</p>${architecturePrinciples()}<article class="panel architecture-capability"><h2>架构声明不等于运行能力</h2><p>模型采用统一接口，但当前仓库契约标明外部运行时未捆绑。实际可用性、输入条件和许可须由对应业务流程判断。</p><div class="button-row"><button class="secondary" data-action="show-model-runtime-boundary">模型接口与运行能力</button><button class="secondary" data-action="jump-workspace-anchor" data-anchor-id="architecture-principles">查看运行原则</button><button class="secondary" data-action="jump-workspace-anchor" data-anchor-id="architecture-modules">查看模块职责</button></div></article>`;
}
export function architectureModules(): string {
  const modules = [["桌面客户端层","负责页面显示和操作响应"],["业务流程层","串联录入、推演、复盘和分析"],["数据规则层","统一管理赛事、球队、球员和比赛数据"],["模型计算层","定义模型计算契约；运行时实现与许可分别校验"],["数据库访问层","负责保存数据和自动升级数据库结构"]];
  return `<article class="panel"><h2>底层模块职责</h2><p>模块用于分离维护责任；本页不执行模型、迁移或连接测试。</p><details class="inline-details"><summary>展开模块说明</summary><div class="domain-list user-domain-list">${modules.map(([name,description])=>`<div><b>${name}</b><span>${description}</span></div>`).join("")}</div></details></article>`;
}

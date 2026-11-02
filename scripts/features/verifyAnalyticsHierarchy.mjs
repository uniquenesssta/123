import assert from 'node:assert/strict';
import {loadTeamsProduction} from './loadTeamsProduction.mjs';
import {analyticsFixture} from './analyticsFixtures.mjs';
import {postmatchFixture} from './postmatchFixtures.mjs';
/** Replace obsolete source-shape assumptions with executable checks for every original stage and gate. */
export async function verifyAnalyticsHierarchy(root){
 const load=loadTeamsProduction(root),{AnalyticsFeature,analyticsStorage,analyticsPage}=load('src/features/analytics/index.ts'),{PostmatchFeature,postmatchRecovery}=load('src/features/postmatch/index.ts');
 const x=analyticsFixture(AnalyticsFeature,analyticsStorage),postmatch=postmatchFixture(PostmatchFeature,postmatchRecovery),state=load('scripts/design-refresh/fixtureData.ts').state;
 await x.feature.refresh();await postmatch.feature.refresh();let count=0;
 for(const [id,label]of [['history','历史样本'],['model','完整分析'],['quality','质量与审核'],['lifecycle','受控校准']]){
  x.feature.setStep(id);const html=analyticsPage(state,x.feature,postmatch.feature);
  assert.ok(html.includes('data-analytics-step="'+id+'"'),'Original named stage is still reachable: '+id);count++;
  assert.ok(html.includes(label),'Stage retains its visible meaning: '+label);count++;
  assert.ok(html.includes('id="analysis-'+id+'-step"'),'Activating original stage renders its actual content target: '+id);count++;
  assert.equal((html.match(/aria-current="step"/g)??[]).length,1,'Exactly one stage owns active state');count++;
  assert.ok(html.includes('data-workspace-scroll-key="analytics-stage"'),'Current content has the remaining-height scroll owner');count++;
 }
 assert.ok(x.feature.gates.history&&x.feature.gates.analysis&&x.feature.gates.quality&&x.feature.gates.review);count++;
 x.history(0);assert.equal(x.feature.gates.history,false);assert.equal(x.feature.gates.analysis,false);count+=2;x.history(1);
 x.latest.sample_size=0;await x.feature.refresh();assert.equal(x.feature.gates.analysis,false);count++;
 x.latest=x.overview();x.latest.data_quality.critical=1;await x.feature.refresh();assert.equal(x.feature.gates.quality,false);count++;
 x.latest=x.overview();x.suggestions.push(x.suggestion());await x.feature.refresh();assert.equal(x.feature.gates.review,false);count++;
 x.suggestions=[];x.latest=x.overview('qa-scoped','qa-competition');await x.feature.refresh();assert.equal(x.feature.gates.quality,false);assert.equal(x.feature.gates.review,false);count+=2;
 x.feature.setStep('lifecycle');assert.ok(analyticsPage(state,x.feature,postmatch.feature).includes('data-action="check-parameter-lifecycle-readiness"'),'Lifecycle remains independently gated, never marked completed by global summary');count++;
 console.log('Executable Analytics hierarchy and original chain obligations passed: '+count);return count;
}

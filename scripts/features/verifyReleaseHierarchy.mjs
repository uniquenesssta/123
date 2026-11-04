import assert from 'node:assert/strict';import {loadTeamsProduction} from './loadTeamsProduction.mjs';import {releaseFixture} from './releaseFixtures.mjs';
export async function verifyReleaseHierarchy(root){const load=loadTeamsProduction(root),{ReleaseFeature,releaseStorage,releaseFeaturePage}=load('src/features/release/index.ts'),x=releaseFixture(ReleaseFeature,releaseStorage),state=load('scripts/design-refresh/fixtureData.ts').state;let count=0;
 const report=x.seed();await x.feature.refresh();await x.feature.select(report.id);
 for(const [id,label]of [['overview','总览'],['chain','全链路'],['performance','性能'],['security','安全'],['cost','成本'],['history','历史']]){const html=releaseFeaturePage(state,x.feature,id,true);assert.ok(html.includes('data-section-id="'+id+'"'));assert.ok(html.includes('>'+label+'<'));assert.equal((html.match(/aria-current="page"/g)??[]).length,1);assert.ok(html.includes('release-viewport'));count+=4;}
 const html=releaseFeaturePage(state,x.feature,'overview',false);assert.ok(html.includes('release-inspector')&&html.includes('show-release-acceptance-json'));count++;
 assert.ok(!html.includes('panel module-sidebar'));count++;
 const fresh=releaseFixture(ReleaseFeature,releaseStorage),unread=releaseFeaturePage(state,fresh.feature,'overview',true);assert.ok(unread.includes('公开')&&!unread.includes('执行固定 fixture + 真实运行验收'));count++;
 console.log('R14-17 release hierarchy checks passed: '+count);return count;
}

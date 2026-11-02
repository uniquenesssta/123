import type {AnalyticsRuntime} from '../analyticsPorts';import {CandidateContext} from './candidateContext';import {AbilityDecisions} from './abilityDecisions';import {ParameterLifecycle} from './parameterLifecycle';
export class CandidateFeature {
 readonly context:CandidateContext;readonly ability:AbilityDecisions;readonly parameters:ParameterLifecycle;
 constructor(runtime:AnalyticsRuntime){this.context=new CandidateContext(runtime);this.ability=new AbilityDecisions(this.context);this.parameters=new ParameterLifecycle(this.context);}
 reset(){this.context.reset();}
 async recover(id:string):Promise<boolean>{const c=this.context,receipt=c.receipt(id);if(receipt?.kind!=='candidate'||c.state.busy)return false;const current=c.capture(),done=c.begin();try{const ok=receipt.operation.startsWith('ability-')?await this.ability.recover(receipt):await this.parameters.recover(receipt);if(current()&&ok)c.outcome('已按原候选查回结果，没有重复写入');return ok;}catch(e){if(current())c.fail(e,true);return false;}finally{done();}}
}

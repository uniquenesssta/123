import {iconSlot} from '../../ui/icons/iconSlot';
import {DialogController,dialogView} from '../../ui/overlays/dialog';
import {EventScope} from '../../app/lifecycle';
import {buttonView} from '../../ui/primitives/button';
import {escapeHtml} from '../../components/format';
/** Read-only S11 report surface; shared dialog owns modality, focus and native scrolling. */
export class ReleaseDetails {
 private host:HTMLDivElement|null=null;private events:EventScope|null=null;private dialog:DialogController|null=null;
 show(title:string,value:unknown){this.destroy();const host=this.host=document.createElement('div');host.className='release-report-host';document.body.append(host);
  const dialog=this.dialog=new DialogController(host,()=>document.querySelector<HTMLElement>('.release-page [data-action=show-release-acceptance-json],.release-page [data-action=release-scope]'),()=>this.destroy());
  this.events=new EventScope();this.events.listen(host,'click',event=>{if((event.target as Element).closest('[data-release-close]')){event.preventDefault();event.stopPropagation();this.destroy();}else if(dialog.handleBackdrop(event))event.stopPropagation();});this.events.listen(host,'cancel',event=>{if(dialog.handleCancel(event))event.stopPropagation();},{capture:true});this.events.listen(host,'keydown',event=>{if(dialog.handleKeydown(event))event.stopPropagation();},{capture:true});
  dialog.restore(host,dialogView({title,description:'原报告与证据只读；fixture版本不证明私有模型已执行。',className:'release-report-dialog',body:`<pre class="release-report-json">${escapeHtml(JSON.stringify(value,null,2))}</pre>`,actions:buttonView({label:'关闭',variant:'primary',attributes:{'data-release-close':'true'}}),closeAttributes:{'data-release-close':'true',autofocus:true}}).replace('<header class="ui-dialog-header">','<header class="ui-dialog-header"><span class="release-report-icon">'+iconSlot('info',16)+'</span>'));
 }
 destroy(){this.dialog?.close();this.dialog=null;this.events?.destroy();this.events=null;this.host?.remove();this.host=null;}
}

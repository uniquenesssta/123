import {mountNumberInput} from '../../ui/primitives/number-input';
export class ReleaseControls {
 private controls:Array<{destroy():void}>=[];
 mount(root:HTMLElement){this.destroy();root.querySelectorAll<HTMLElement>('.release-page [data-ui-primitive="number-input"]').forEach(node=>this.controls.push(mountNumberInput(node)));}
 destroy(){for(const control of this.controls)control.destroy();this.controls=[];}
}

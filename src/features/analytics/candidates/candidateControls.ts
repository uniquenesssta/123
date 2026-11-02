import {mountSelect} from '../../../ui/primitives/select';
export class CandidateControls {private controls:Array<{destroy():void}>=[];mount(root:HTMLElement){this.destroy();root.querySelectorAll<HTMLElement>('.candidate-workbench [data-ui-primitive="select"]').forEach(node=>this.controls.push(mountSelect(node)));}destroy(){for(const control of this.controls)control.destroy();this.controls=[];}}

import {mountSelect} from '../../ui/primitives/select';
import {mountNumberInput} from '../../ui/primitives/number-input';
export class PostmatchControls {
  private mounted:Array<{destroy():void}>=[];
  mount(root:HTMLElement):void{this.destroy();root.querySelectorAll<HTMLElement>('.postmatch-workbench [data-ui-primitive="select"]').forEach(node=>this.mounted.push(mountSelect(node)));root.querySelectorAll<HTMLElement>('.postmatch-workbench [data-ui-primitive="number-input"]').forEach(node=>this.mounted.push(mountNumberInput(node)));}
  destroy():void{for(const controller of this.mounted)controller.destroy();this.mounted=[];}
}

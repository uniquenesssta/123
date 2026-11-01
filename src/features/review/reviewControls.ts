import { mountSelect } from '../../ui/primitives/select';
import { mountNumberInput } from '../../ui/primitives/number-input';
/** Owns only the live primitive controllers; domain values remain Feature/Facts owned. */
export class ReviewControls {
    private controls: Array<{
        destroy(): void;
    }> = [];
    mount(root: HTMLElement): void {
        this.destroy();
        root.querySelectorAll<HTMLElement>('.review-page [data-ui-primitive="select"]').forEach(node => this.controls.push(mountSelect(node)));
        root.querySelectorAll<HTMLElement>('.review-page [data-ui-primitive="number-input"]').forEach(node => this.controls.push(mountNumberInput(node)));
    }
    destroy(): void { for (const controller of this.controls)
        controller.destroy(); this.controls = []; }
}

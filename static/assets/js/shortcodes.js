//
// Last Modification: 2024-07-15 17:54:15
//

class Shortcodes {

    constructor(parameters = {
        "start_element": "body"
    }) {
        this.startElement = parameters["start_element"];
    }

    async getData(url) {
        try {
            const response = await fetch(url);
            if (!response.ok) {
                throw new Error(`Response status: ${response.status}`);
            }

            return await response.text();
        } catch (error) {
            console.error(error.message);
            return null;
        }
    }

    swapContent(clone, target, swap) {
        if (swap === 'inner') { // default swap option
            target.replaceChildren(...clone.childNodes);
            return;
        }

        // Replaces this Element in the children list of
        // its parent with a set of Node or string objects
        if (swap === 'outer') {
            target.replaceWith(...clone.childNodes);
            return;
        }

        throw new Error(`swap "${swap}" attribute not supported`);
    }

    async parse() {
        const targetNode = document.getElementsByTagName(this.startElement);
        const shortcodes = targetNode[0].querySelectorAll('[data-shortcode]');
        for (const target of shortcodes) {
            const url = target.getAttribute('data-shortcode');

            const swap = target.hasAttribute('data-swap') ? target.getAttribute('data-swap') : 'outer';

            const data = await this.getData(url);
            console.log(data);

            const helper = document.createElement('div');
            helper.innerHTML = data;
            this.swapContent(helper, target, swap);
        }
    }
}

const sc = new Shortcodes();
sc.parse();
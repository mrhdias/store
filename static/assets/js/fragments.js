//
// fragments.js
// Author: Henrique Dias
// Last Modification: 2024-06-14 11:30:01
//

(function (global, factory) {
    if (typeof exports === 'object' && typeof module !== 'undefined') {
        module.exports = factory();
    } else if (typeof define === 'function' && define.amd) {
        define(factory);
    } else {
        global.Fragments = factory();
    }
}(this, (function () {
    'use strict';

    class Fragments {

        constructor(parameters = {
            "start_element": "body"
        }) {
            // https://developer.mozilla.org/en-US/docs/Web/API/Element#events
            // https://developer.mozilla.org/en-US/docs/Web/API/Window/load_event
            // Some events to test
            this.triggers = new Set([
                'click',
                'init' // custom
            ]);
            this.methods = new Set([
                'get',
                'post',
                'put',
                'patch',
                'delete',
            ]);
            this.startElement = parameters["start_element"];
        }

        async makeRequest(properties) {
            try {
                // https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API/Using_Fetch
                const response = await fetch(properties.action, {
                    method: properties.method,
                    mode: "cors", // no-cors, *cors, same-origin
                    cache: "no-cache", // *default, no-cache, reload, force-cache, only-if-cached
                    credentials: "same-origin" // include, *same-origin, omit
                });

                let result = {
                    data: undefined,
                    ok: response.ok,
                    status: response.status
                };

                const fragment = await response.text();
                result.data = fragment;

                return result;
            } catch (error) {
                return {
                    data: undefined,
                    ok: false,
                    status: 0
                };
            }
        }

        swapContent(clone, target, swap) {
            if (target === null) {
                console.warn('no target to swap');
                return
            }

            // replaces the existing children of a Node
            // with a specified new set of children.
            if (swap === 'inner') { // default swap option
                target.replaceChildren(...clone.childNodes);
                return;
            }

            // replaces this Element in the children list of
            // its parent with a set of Node or string objects
            if (swap === 'outer') {
                target.replaceWith(...clone.childNodes);
                return;
            }

            // inserts a set of Node or string objects in the children
            // list of this Element's parent, just before this Element.
            if (swap === 'before') {
                target.before(...clone.childNodes);
                return;
            }

            // inserts a set of Node or string objects in the children
            // list of the Element's parent, just after the Element.
            if (swap === 'after') {
                target.after(...clone.childNodes);
                return;
            }

            // inserts a set of Node objects or string
            // objects before the first child of the Element.
            if (swap === 'prepend') {
                target.prepend(...clone.childNodes);
                return;
            }

            // inserts a set of Node objects or string
            // objects after the last child of the Element.
            if (swap === 'append') {
                target.append(...clone.childNodes);
                return;
            }

            throw new Error(`swap "${swap}" attribute not supported`);
        }

        async processEvent(eventTarget, properties) {
            const result = await this.makeRequest(properties);
            if (result.ok) {
                // console.log('Data:', result.data);

                const helperElem = document.createElement('div');
                helperElem.innerHTML = result.data;

                const target = (properties.target === 'this') ? eventTarget : document.querySelector(properties.target);

                const clone = helperElem.cloneNode(true);

                this.findActions(clone);

                this.swapContent(clone, target, properties.swap);

            } else {
                console.error('Error:', result.status);
            }
        }

        findActions(element) {
            const actions = element.querySelectorAll('[data-action]');
            for (const action of actions) {
                // console.log('action:', action);
                const properties = {
                    'action': action.getAttribute('data-action'),
                    'trigger': (() => {
                        if (action.hasAttribute('data-trigger')) {
                            return action.getAttribute('data-trigger');
                        }
                        return 'click';
                    })(),
                    'method': (() => {
                        if (action.hasAttribute('data-method')) {
                            return action.getAttribute('data-method');
                        }
                        return 'get';
                    })(),
                    'target': (() => {
                        if (action.hasAttribute('data-target')) {
                            return action.getAttribute('data-target');
                        }
                        return 'this';
                    })(),
                    'swap': (() => {
                        if (action.hasAttribute('data-swap')) {
                            return action.getAttribute('data-swap');
                        }
                        return 'inner';
                    })()
                };

                // test if trigger is allowed
                if (!this.triggers.has(properties.trigger)) {
                    console.warn(`The "${properties.trigger}" trigger is not allowed yet!`);
                    continue;
                }

                // test if method is allowed
                if (!this.methods.has(properties.method)) {
                    console.warn(`The "${properties.method}" method is not allowed yet!`);
                    continue;
                }

                if (properties.trigger === 'init') {
                    // https://developer.mozilla.org/en-US/docs/Web/Events/Creating_and_triggering_events
                    // Create 'init' event
                    action.addEventListener('init', async (e) => {
                        e.preventDefault();
                        try {
                            await this.processEvent(e.currentTarget, properties);
                        } catch (error) {
                            console.log(`Error for "${properties.action}": ${error}`);
                        }
                    });
                    // after the addEventListener to associate the target
                    // and currentTarget, if not it is null.
                    const event = new CustomEvent('init', {
                        bubbles: true,
                        cancelable: true
                    });
                    action.dispatchEvent(event);

                    continue;
                }

                action.addEventListener(properties.trigger, async (e) => {
                    e.preventDefault();
                    try {
                        await this.processEvent(e.currentTarget, properties);
                    } catch (error) {
                        console.log(`Error for "${properties.action}": ${error}`);
                    }
                });

            }
        }

        init() {
            const targetNode = document.getElementsByTagName(this.startElement);
            this.findActions(targetNode[0]);
        }
    }

    return Fragments;
})));


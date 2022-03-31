# Concepts

reView is a simple library to create single page application. 
The API itself is similar to the react API and for this reason some concepts like hooks and functional components are similar.

To minimalize the DOM manipulation reView use a virtual DOM that is synchronized with the real DOM. 
Every node in the virtual DOM is represented by a `VNode` that will be materialized in a real DOM element only when required.

There is 3 kind of `VNode`:
- a `Element` that represent a standard DOM element and corresponds to a DOM node created with [createElement](https://developer.mozilla.org/en-US/docs/Web/API/Document/createElement) (eg: `div`, `p`, `button`, etc...)
- a `Text` that represent a simple string displayed in the DOM and corresponds to a DOM node created with [createTextNode](https://developer.mozilla.org/en-US/docs/Web/API/Document/createElement) 
- a `Component` that is a main reView building block represented by a function that returns a `VNode`
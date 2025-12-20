# Work Scheduling

## The Graph Nature of Darkmatter Documents

When we parse a Darkmatter document, we first interrogate the graph of resources that are needed to complete the rendering of this document.

![document graph](../images/reference-tree.png)

The picture above illustrates the graph based nature of a Darkmatter document.

- the recursive structure branches out until it reaches leaf branches of the tree
- leaf nodes will be one of the following:
    - a local PDF document has no ability to express external references to bring in
    - a Markdown/Darkmatter document which has no external resource references
        - this _could_ include a Darkmatter
    - an _optimized_ image via [Smart Image](../design/smart-image.md)
- all other nodes have dependencies:
    - **Image References** -- whether hosted locally or referred to remotely -- must go through the optimization steps detailed in the [Smart Image](../design/smart-image.md) specification.
        - The initial reference is not a leaf node but instead the set of optimized images are the leaf nodes
    -


## What is Work Scheduling and Why Do We Need It?



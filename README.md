# Orchestrator

[![DOCS](https://img.shields.io/badge/docs-orchestrator?style=for-the-badge&logo=docs.rs&labelColor=%23567&color=%233A3)](https://handzcz.github.io/orchestrator/)

A crate that provides pipelines and an orchestrator.

It provides traits `Pipeline` and `Orchestrator`.
It also provide generic implementations (`GenericPipeline`, `GenericOrchestrator`) of these traits.
To use the generic types you need to implement the `Node` trait and mostly follow the compilers nagging.

For further details about `GenericPipeline`, `GenericOrchestrator` or `Node` look at docs.
And if you are looking for an example you can find some in examples directory or in docs,
where `Pipeline`, `Orchestrator` and `Node` have code examples as well as their generic implementations.

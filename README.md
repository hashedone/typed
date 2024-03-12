# Typed Programming Language

Typed is a programming language meant to be Wasm first-class language. It is designed to be a general purpose scripting programming language, to be used in any context that Runs Wasm binaries.

## Principles

`Typed` is inspired mostly by `Rust` and `Haskell` languages, and also by Bartosz Milewskis [category theory](https://www.youtube.com/watch?v=I8LbkfSSR58&list=PLbgaMIhjbmEnaH_LTkxLI7FMa2HsnawM_) lectures.

The first usage typed is meant for are CosmWasm smart contracts, but typed is not meant as SC DSL like Solidity. Still, as it is the fundamental application of the language, it may influence its development.

The principles are:
* Very strong and verbose typesystem, allowing to create solid and safe APIs
* As much transparent typesystem as possible - ideally everything should be elideable (but possible to be explicit)
* Possible to use it in pure functional way
* Should distinguish at least `pure` code (no side-effects, matematical pureness), `unpure` code - possible side-effects, maybe non deterministic, and `unsafe` code - possible memory unsafety
* Possible to hide `unsafe`/`unpure` code behind `pure`/`safe` facade - programmer doesn't need to care if the vector allocates antyhing, as long, as it behaves as just `Foldable`

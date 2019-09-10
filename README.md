# Why yet another language?
You don't need it. It is actually a whim to take a shot on this.

There are two reasons of `Typed` existence:
1. I was thinking about "idiomatic" `tensorflow` bindings in Rust, however I found out, that there is no reasonable dependent types equivalent in Rust. Actually I knew it before, but it never hurt me this much before. I postponed try unless at least `const generics` would be useable and ~stable (at least in nightly).
2. I was looking at `tensorflow` bindings in Haskell (to better undestand Haskell idioms), and I realised, that FFI interface in Haskell ruins its purity - function with side effects can be straightly called as pure function. It is recommended to "mark", or rather "enrich" the return value with `IO` monad, but it is completely optional.

## Principles
`Typed` is inspired mostly by `Rust` and `Haskell` languages, and also by Bartosz Milewskis [category theory](https://www.youtube.com/watch?v=I8LbkfSSR58&list=PLbgaMIhjbmEnaH_LTkxLI7FMa2HsnawM_) lectures. There may be an influence of `C++`, in particular template (and template metaprogramming).

The principles are:
* Very strong and verbose typesystem, allowing type transormations
* As much transparent typesystem as possible - ideally everything should be elideable (but possible to be explicit)
* HW aware, "zero overhead" - defienetely no GC, should allow to use HW directly in some way
* Possible to use it in pure functional way
* Should distinguish at least `pure` code (no side-effects, matematical pureness), `unpure` code - possible side-effects, maybe non deterministic, and `unsafe` code - possible memory unsafety
* Possible to hide `unsafe`/`unpure` code behind `pure`/`safe` facade - programmer doesn't need to care if the vector allocates antyhing, as long, as it behaves as just `Foldable`

### Problems

* Handling IO - possible approaches are at least something simmilar to Haskell `IO` monad, or algebraic effects

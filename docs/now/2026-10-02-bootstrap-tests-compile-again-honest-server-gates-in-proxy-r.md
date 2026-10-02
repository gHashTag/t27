# NOW -- bootstrap tests compile again: honest server gates in proxy.rs, E0716 in corpus_unresolved (2026-10-02)

## bootstrap tests compile again: honest server gates in proxy.rs, E0716 in corpus_unresolved (Closes #5448)

- proxy.rs gated its axum/tokio code on any(feature = "server", test); those crates are optional and enabled only by the server feature, so a plain cargo test switched the code on without its dependencies and the t27c bin test target failed to compile, halting every cargo test run on master.
- Everything that touches axum, tokio, AppState or Session is now gated on feature = "server" alone. The two token parsers take HeaderMap/Uri from hyper (non-optional, same http 1.x types), so they and their unit tests still type-check and run in a default cargo test.
- Restored builder.build(connector) before .request() in proxy_to_container; #4758 had dropped it, which also broke cargo build --features server used by both Dockerfiles.
- tests/corpus_unresolved.rs: bind Command::new to a let before chaining .args(), fixing E0716 (temporary dropped while borrowed).

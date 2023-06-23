# Risc0 based zkLambda demo

## Building

```
    cargo install wasm-pack
    wasm-pack build verifier --target web
    cargo build -r
```

## Running

```
    ./target/release/prover
    http://localhost:8090/
```
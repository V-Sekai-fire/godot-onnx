# Testing

## Property tests (Rust, no Godot)

The crate uses [proptest](https://docs.rs/proptest) for property-based tests of the identity and matmul models:

```bash
cargo test
```

- **identity_preserves_input**: for any `[f32; 3]`, the identity model output equals the input.
- **matmul_matches_reference**: for any 2×3 and 3×2 matrices, the matmul model output matches a reference implementation within tolerance.

Tests live in `src/prop_tests.rs` and require the same ONNX files in `sample/models/` (see [Usage](USAGE.md#test-models-identity--matmul)). To increase the number of random cases:

```bash
PROPTEST_CASES=500 cargo test
```

## Godot sample

Open `sample/` in Godot 4.4+ and run the main scene. It loads identity and matmul, runs one inference each, and prints results to the console.

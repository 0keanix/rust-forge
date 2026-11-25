---
name: rust-domain-ml
description: Guides building and changing Rust machine-learning and inference projects that use candle (candle-core, candle-nn, candle-transformers), burn or llama.cpp bindings (llama-cpp-2) - choosing between them, safetensors and GGUF model formats, CPU, Metal and CUDA feature flags with Metal on Apple Silicon and a CPU fallback, memory-mapped weights, tokenizer handling, batching, and deterministic tests with tiny models or recorded fixtures instead of real downloads, with benchmarks kept out of the test suite. Use it whenever a Rust project depends on candle-core, burn or llama-cpp-2, or the user asks to load a model, run inference, embed text, add a GPU backend or test model code, even if they only say "the model", "embeddings" or "inference". Complements the core rust-forge skill, which still owns the working loop and gates.
metadata:
  forge-verified: candle-core=0.11.1,burn=0.22.1,llama-cpp-2=0.1.161
---

# rust-domain-ml

Domain guidance for ML inference and training in Rust. The core `rust-forge` skill owns the working loop, the gates,
the crate policy and compiler-error strategies; follow it and use this skill only for what is specific to ML crates.

## 1. Pin the version first

Run `forge-tools domains` and read the `ml` line: `crate=<candle-core|burn|llama-cpp-2> version=<x>`. Read docs.rs,
the official guides and examples (the candle repository examples, the Burn Book, the llama-cpp-2 examples) for exactly
that version before writing code. These crates are pre-1.0 and reshape APIs and feature names between minor
releases, and version drift is the main source of wrong ML code. If the output says `version=unknown`, the lock file
is missing; take the requirement from `Cargo.toml` and resolve it before relying on details.

The API and feature names below were checked against candle-core and candle-nn 0.11.1, tokenizers 0.23.2, burn
0.22.1 and llama-cpp-2 0.1.161. For any other version, confirm each name in that version's docs before using it.

## 2. Choosing the stack

- candle: a small tensor library with ready model implementations in `candle-transformers`. Pick it for inference of
  Hugging Face models from safetensors (embeddings, small transformers) where you want plain Rust and few
  dependencies.
- burn: a full framework with swappable backends and training (autodiff, optimizers). Pick it when you train or
  fine-tune, or need the same model code on several backends.
- llama-cpp-2: bindings to llama.cpp, built from C/C++ sources. Pick it for quantized GGUF LLMs where llama.cpp's
  kernels and quantization matter more than a pure-Rust build; expect a C/C++ toolchain and cmake at build time.
- Keep to the crate the project already uses. Mixing two stacks doubles build time and tensor conversions.

## 3. Model formats

- safetensors: the default for candle and burn. Files hold named tensors and a header, so loading is a lookup by
  name and needs no code execution, unlike pickled PyTorch checkpoints; prefer it over `.pth`/`.bin`.
- GGUF: the llama.cpp format, carrying quantized weights and metadata (tokenizer, architecture) in one file. Use it
  with llama-cpp-2; candle reads it only for its quantized model implementations.
- Check tensor names against the model's own implementation (or list them from the file) instead of guessing:
  a prefix mismatch (`bert.` vs none) is the most common load failure.
- Keep model paths configurable (flag, env var, config). Never hard-code a cache directory or download in library
  code; let the caller fetch weights so tests can pass a fixture path.

## 4. Devices and feature flags

- Accelerators are cargo features, not runtime switches: candle-core and candle-nn have `metal`, `cuda`, `mkl` and
  `accelerate`; burn has backend features such as `ndarray`, `cpu`, `wgpu`, `metal` and `cuda`; llama-cpp-2 has
  `metal`, `cuda` and `vulkan`. The defaults build CPU only.
- Expose them as features of your own crate (for example `metal = ["candle-core/metal", "candle-nn/metal"]`) and
  never enable them by default. A default GPU feature breaks the build on every machine without that SDK, including
  CI.
- Choose the device at runtime with a CPU fallback. In candle 0.11.1, `Device::metal_if_available(0)` and
  `Device::cuda_if_available(0)` return `Device::Cpu` when the feature is not compiled in
  (`candle_core::utils::metal_is_available()` only reports the feature). With the feature on they try the device and
  return an error when none is found, so also fall back to `Device::Cpu` on that error: Metal on Apple Silicon, CPU
  everywhere else.
- Tensors on different devices cannot be combined; create inputs on the model's device, and move results to CPU
  only at the boundary (for example before converting to `Vec<f32>`).
- Watch dtypes: half precision (`F16`, `BF16`) is fast on GPUs but not supported everywhere on CPU; load in `F32` on
  CPU unless you checked support.

## 5. Memory-mapped weights

- Memory-map large weight files instead of reading them into a `Vec`: the OS pages weights in on demand, start-up is
  faster and the process does not hold two copies. In candle-nn 0.11.1 this is
  `unsafe { VarBuilder::from_mmaped_safetensors(&paths, dtype, &device) }`.
- The `unsafe` is real: the mapping is undefined behaviour if the file changes while mapped. Keep it in one small
  function with a `// SAFETY:` comment that states the file is opened read-only and not modified while the model
  lives, and keep the rest of the code safe.
- For tests and small in-memory models use the safe constructors (`VarBuilder::from_slice_safetensors`,
  `VarBuilder::from_tensors`) so tests need no file and no `unsafe`.

## 6. Tokenizers

- Load the tokenizer that ships with the model (`tokenizer.json`), never one for a similar model: a vocabulary
  mismatch produces plausible but wrong output, not an error. With the `tokenizers` crate this is
  `Tokenizer::from_file`; its default features pull in C code (`onig`), so check whether the project already builds
  it.
- Set padding and truncation explicitly (`with_padding`, `with_truncation`) to the model's maximum length, and keep
  the attention mask: pooling or attention over padding tokens silently changes results.
- Keep tokenization separate from the model forward pass so either can be tested alone.

## 7. Batching and post-processing

- Batch inputs into one tensor (`encode_batch`, then stack ids and masks) instead of looping per item: per-call
  overhead dominates small models, especially on GPU.
- Pad to the longest item in the batch, not a global maximum, and bound the batch size so memory stays predictable.
- For embeddings, pool with the attention mask (mean over real tokens only) and L2-normalize each vector; guard the
  division against a zero norm. Normalized vectors make cosine similarity a dot product, and callers rely on it.
- Return plain Rust types (`Vec<Vec<f32>>`) from the public API so callers do not depend on the tensor crate.

## 8. Deterministic tests without downloads

- Tests must not touch the network or a model cache: downloads make CI slow, flaky and dependent on a third-party
  service. Never call hub or `from_pretrained` style loaders in tests.
- Build a tiny model in the test instead: a few-dimensional config with weights from fixed values, saved with
  `candle_core::safetensors::save` to a temp file (or passed to `VarBuilder::from_tensors`), so the full load path
  runs in milliseconds.
- For checks against a real model, record its outputs once (input ids, a few output values) into a small fixture
  file committed to the repo, and compare with a tolerance (`abs diff < 1e-4`), never with exact float equality.
- Test the properties that do not depend on weights: output shape, unit norm after normalization, identical input
  giving identical output, and that padding does not change an item's embedding.
- Run tests on CPU without features. GPU code paths are checked by building with the feature
  (`cargo check --features metal` on a Mac), not by the default test suite, so the gates pass on any machine.

## 9. Benchmarks

Keep throughput and latency measurements out of `cargo test`: put them in `benches/` (criterion, or the core skill's
benchmark guidance) and run them on demand. Timing assertions in tests fail on loaded CI machines and real models are
too large for the suite.

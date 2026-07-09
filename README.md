# pedersen-py

The whole family of widely-known **Pedersen hash functions** built from **one skeleton**, over arkworks curves — with Python bindings.

Every concrete hash is the same multi-scalar multiplication

```text
H(m) = O  +  Σ_k  contribute(chunk_k, power_k)
```

and the differences between arkworks, circom/iden3 and Zcash Sapling are just *which* components you plug into that one engine.

## Workspace

| Crate | What it is |
|-------|------------|
| [`crates/pedersen-kit`](crates/pedersen-kit) | Pure-Rust core: the generic engine + four pluggable axes (`Encoding`, `BitLayout`, `Generators`, `OutputEncoding`) and ready-made instances. Publishable, reusable. |
| [`crates/pedersenpy`](crates/pedersenpy) | PyO3 bindings (`import pedersenpy`) exposing the concrete hashers to Python. |

## Hashers

| | Rust (`pedersen-kit`) | Python (`pedersenpy`) | Compatibility |
|---|---|---|---|
| Unsigned windows (Baby Jubjub) | `instances::BabyJubjubPedersen` | `BabyJubjubPedersen(segments, bits_per_window)` | arkworks `pedersen::CRH` |
| Bowe–Hopwood (Jubjub) | `instances::JubjubBoweHopwood` | `JubjubBoweHopwood(segments, chunks_per_segment)` | arkworks `bowe_hopwood::CRH` |
| circom / iden3 (Baby Jubjub) | `circom::BabyJubjubCircom` | `CircomPedersen()` | **byte-exact** `circomlibjs pedersenHash` |
| Zcash Sapling (Jubjub) | `zcash::JubjubSapling` | `ZcashPedersen(personalization=b"Zcash_PH")` | **byte-exact** Zcash Sapling |

The circom/Zcash instances derive their spec generators once, cache them (memoized, reusable), ship the first 6 as baked constants, and fall back to BLAKE for arbitrary length. A test re-derives the baked tables via the hash to prevent drift.

## Quick start

**Rust**
```rust
use pedersen_kit::circom::BabyJubjubCircom;

let mut h = BabyJubjubCircom::new();
let digest: [u8; 32] = h.hash(b"Hello"); // matches circomlibjs
```

**Python** (built with [maturin](https://www.maturin.rs))
```python
import pedersenpy
h = pedersenpy.CircomPedersen()
digest = h.hash(b"Hello")                 # bytes, matches circomlibjs

z = pedersenpy.ZcashPedersen()
u = z.hash(b"Hello")                       # 32-byte LE u-coordinate, matches Zcash Sapling
```

## Features (`pedersen-kit`)

- `circom`, `zcash` — the byte-compatible instances (on by default; pull `blake-hash` / `blake2s_simd`).
- `--no-default-features` — lean core only, no blake dependencies.

## ERC-2494 Baby Jubjub

The Baby Jubjub curve comes from [`ark-babyjubjub`](https://github.com/arkworks-rs/algebra/pull/1123), which is **not yet published**. Until it merges, the workspace pins it (and unifies the arkworks graph via `[patch.crates-io]`) to that PR commit. `pedersenpy` is meant to be installed **from a built wheel**, which is self-contained.

## Testing

```bash
cargo test -p pedersen-kit                      # incl. circom/zcash vectors + drift tests
cargo test -p pedersen-kit --no-default-features # lean core (arkworks parity only)
maturin build -m crates/pedersenpy/Cargo.toml   # build the Python wheel
```

Reference vectors are validated against arkworks (`ark-crypto-primitives`), `circomlibjs`, and `zcash/zcash-test-vectors`.

## License

MIT OR Apache-2.0

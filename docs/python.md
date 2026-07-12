# Python guide

**`pedersenpy`** exposes the concrete hashers to Python via PyO3. It ships PEP 561 type information (`py.typed` + stubs), so `mypy`/`pyright` and IDEs get full typing.

## Installing

```bash
pip install pedersenpy
```

Available on [PyPI](https://pypi.org/project/pedersenpy/). Wheels are self-contained (`abi3`, Python ≥ 3.9) — no Rust toolchain needed to install.

To build from source instead (see [Publishing](publishing.md)):

```bash
maturin build --release -m crates/pedersenpy/Cargo.toml
pip install target/wheels/pedersenpy-*.whl
```

## Usage

```python
import pedersenpy

# circom / iden3 — byte-identical to circomlibjs pedersenHash
c = pedersenpy.CircomPedersen()
c.hash(b"Hello").hex()
# '0e90d7d613ab8b5ea7f4f8bc537db6bb0fa2e5e97bbac1c1f609ef9e6a35fd8b'

# Zcash Sapling — 32-byte little-endian u-coordinate
z = pedersenpy.ZcashPedersen()                 # default personalization b"Zcash_PH"
u = z.hash(b"Hello")
int.from_bytes(u, "little")
# 8754254972755604884333948367738998890971419059392001151429652007230018821080

# arkworks-shaped instances (Deterministic generators)
pedersenpy.BabyJubjubPedersen(64, 4).hash(b"data")
pedersenpy.JubjubBoweHopwood(16, 40).hash(b"data")
```

!!! tip "Reuse instances"
    `CircomPedersen` and `ZcashPedersen` cache their generators. Reuse one instance for many hashes rather than constructing per call.

## API reference

### `CircomPedersen()`
circom / iden3-compatible Baby Jubjub Pedersen hash.

- `hash(data: bytes) -> bytes` — the 32-byte packed point (circomlibjs `packPoint`).

### `ZcashPedersen(personalization: bytes | None = None)`
Zcash Sapling Pedersen hash over Jubjub. `personalization` must be exactly **8 bytes** (defaults to `b"Zcash_PH"`); anything else raises `ValueError`.

- `hash(data: bytes) -> bytes` — the 32-byte little-endian u-coordinate.

### `BabyJubjubPedersen(segments: int, bits_per_window: int)`
Unsigned-window Pedersen hash over Baby Jubjub (arkworks `pedersen::CRH` shape). Capacity is `segments * bits_per_window` input bits.

- `hash(data: bytes) -> bytes` — the compressed curve point.

### `JubjubBoweHopwood(segments: int, chunks_per_segment: int)`
Bowe–Hopwood Pedersen hash over Jubjub (arkworks `bowe_hopwood::CRH` shape). Capacity is `segments * chunks_per_segment * 3` input bits.

- `hash(data: bytes) -> bytes` — the x-coordinate.

## Type checking

The package is typed. With the stubs installed, this checks clean:

```python
import pedersenpy
digest: bytes = pedersenpy.CircomPedersen().hash(b"x")
```

and `mypy` flags misuse (wrong argument counts, wrong return types). CI runs `mypy` against the built wheel.

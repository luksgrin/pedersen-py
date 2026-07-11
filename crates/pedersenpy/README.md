# pedersenpy

Rust-backed **Pedersen hash functions** for Python — one engine, the whole family of widely-known variants, validated **byte-for-byte** against their references.

`arkworks`, `circomlib`/`iden3`, and `Zcash Sapling` all define "a Pedersen hash," but their outputs disagree. `pedersenpy` implements the shared construction once (in Rust, on [arkworks](https://arkworks.rs) curves) and exposes each ecosystem's hash.

## Install

```bash
pip install pedersenpy
```

Wheels are self-contained (`abi3`, Python ≥ 3.9) — no Rust toolchain needed to install.

## Usage

```python
import pedersenpy

# circom / iden3 — byte-identical to circomlibjs pedersenHash
c = pedersenpy.CircomPedersen()
c.hash(b"Hello").hex()
# '0e90d7d613ab8b5ea7f4f8bc537db6bb0fa2e5e97bbac1c1f609ef9e6a35fd8b'

# Zcash Sapling — 32-byte little-endian u-coordinate
z = pedersenpy.ZcashPedersen()                 # default personalization b"Zcash_PH"
int.from_bytes(z.hash(b"Hello"), "little")
# 8754254972755604884333948367738998890971419059392001151429652007230018821080

# arkworks-shaped instances (deterministic generators)
pedersenpy.BabyJubjubPedersen(64, 4).hash(b"data")
pedersenpy.JubjubBoweHopwood(16, 40).hash(b"data")
```

`CircomPedersen` and `ZcashPedersen` cache their generators — reuse one instance for many hashes.

## Hashers

| Class | Returns | Compatible with |
|---|---|---|
| `CircomPedersen()` | 32-byte packed point | **byte-exact** `circomlibjs pedersenHash` |
| `ZcashPedersen(personalization=b"Zcash_PH")` | 32-byte LE u-coordinate | **byte-exact** Zcash Sapling |
| `BabyJubjubPedersen(segments, bits_per_window)` | compressed point | arkworks `pedersen::CRH` |
| `JubjubBoweHopwood(segments, chunks_per_segment)` | x-coordinate | arkworks `bowe_hopwood::CRH` |

The package ships type stubs (`py.typed`), so `mypy`/`pyright` and editors get full typing.

## Links

- **Documentation:** https://luksgrin.github.io/pedersen-py/
- **Source:** https://github.com/luksgrin/pedersen-py
- **Rust crate (`pedersen-kit`):** the reusable core engine.

## License

MIT

# pedersen-py

**One skeleton for the whole family of widely-known Pedersen hash functions** — over arkworks curves, with Python bindings.

`arkworks`, `circomlib`/`iden3`, and `Zcash Sapling` all define "a Pedersen hash," but their outputs disagree byte-for-byte. This project shows they are the *same* construction with different parameters, implements it **once**, and recovers each ecosystem's hash by plugging in the right components — validated **byte-for-byte** against every reference.

<div class="grid cards" markdown>

- :material-language-rust: **Rust core** — `pedersen-kit`: a generic engine + four pluggable axes.
  [Rust guide →](rust.md)
- :material-language-python: **Python bindings** — `pedersenpy`: the concrete hashers, typed.
  [Python guide →](python.md)
- :material-function-variant: **How it works** — the unifying formula, encodings, curves.
  [Concepts →](concepts.md)
- :material-check-decagram: **Compatibility** — arkworks / circom / Zcash, byte-exact.
  [Compatibility →](compatibility.md)

</div>

## At a glance

| Hasher | Rust (`pedersen-kit`) | Python (`pedersenpy`) | Compatible with |
|---|---|---|---|
| Unsigned windows (Baby Jubjub) | `instances::BabyJubjubPedersen` | `BabyJubjubPedersen` | arkworks `pedersen::CRH` |
| Bowe–Hopwood (Jubjub) | `instances::JubjubBoweHopwood` | `JubjubBoweHopwood` | arkworks `bowe_hopwood::CRH` |
| circom / iden3 (Baby Jubjub) | `circom::BabyJubjubCircom` | `CircomPedersen` | **byte-exact** `circomlibjs pedersenHash` |
| Zcash Sapling (Jubjub) | `zcash::JubjubSapling` | `ZcashPedersen` | **byte-exact** Zcash Sapling |

## Install

=== "Python"

    ```bash
    pip install pedersenpy   # once published; until then, build a wheel (see Publishing)
    ```

    ```python
    import pedersenpy
    pedersenpy.CircomPedersen().hash(b"Hello").hex()
    # '0e90d7d613ab8b5ea7f4f8bc537db6bb0fa2e5e97bbac1c1f609ef9e6a35fd8b'
    ```

=== "Rust"

    ```toml
    [dependencies]
    pedersen-kit = "0.1"   # once published; see Publishing for the current git pin
    ```

    ```rust
    use pedersen_kit::circom::BabyJubjubCircom;
    let mut h = BabyJubjubCircom::new();
    let digest: [u8; 32] = h.hash(b"Hello");
    ```

!!! note "ERC-2494 Baby Jubjub"
    The Baby Jubjub curve comes from [`ark-babyjubjub`](https://github.com/arkworks-rs/algebra/pull/1123), not yet on crates.io. Until it merges, the crate is pinned to that PR commit — see [Publishing](publishing.md).

The project is [MIT licensed](https://github.com/luksgrin/pedersen-py/blob/main/LICENSE).

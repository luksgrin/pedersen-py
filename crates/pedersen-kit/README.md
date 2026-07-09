# pedersen-kit

One Pedersen-hash **skeleton**, from which the widely-known Pedersen hashes are
obtained purely by parameterization. Bottom-up, every member is the same
multi-scalar multiplication built from the same components — only the plug-ins
differ.

```text
H(m) = O  +  Σ_k  contribute(chunk_k, power_k)
```

## The four axes

| Trait | Chooses | Provided components |
|-------|---------|---------------------|
| `Encoding` | chunk size, power spacing, digit map | `Unsigned` (arkworks `pedersen`), `BoweHopwood` (Zcash / arkworks `bowe_hopwood`) |
| `BitLayout` | bytes → bit stream | `LsbFirst` (arkworks `bytes_to_bits`), `MsbFirst` |
| `Generators` | base points + offset | `Deterministic` (reproducible) |
| `OutputEncoding` | group element → output | `WholePoint`, `Compressed`, `XCoordinate`, `XCoordinateBytes` |

A concrete hash is one type: `Pedersen<Curve, Encoding, BitLayout, OutputEncoding>`.

## Reuse of arkworks

- All curve/field arithmetic is `ark-ec` / `ark-ff`; curves are arkworks curve
  crates (`ark-ed-on-bn254` = Baby Jubjub, `ark-ed-on-bls12-381` = Jubjub).
- `Parameters` deliberately mirror `ark_crypto_primitives`' Pedersen parameters,
  so the skeleton reproduces arkworks' `pedersen::CRH` and `bowe_hopwood::CRH`
  **bit-for-bit** given the same generators — proven in `tests/parity.rs`.
- We do **not** reuse arkworks' `setup`/`evaluate` themselves: their generators
  are RNG-sampled (non-reproducible) and their I/O conventions are fixed — i.e.
  exactly the parts that must be parametrizable here. Per the design rule, those
  specific pieces are dropped; the arithmetic underneath is kept.

## In scope

- `BabyJubjubPedersen` — unsigned windows over Baby Jubjub.
- `JubjubBoweHopwood` — signed 3-bit chunks over Jubjub (Zcash-Sapling shape).
- `BabyJubjubBoweHopwood` — Bowe–Hopwood over Baby Jubjub (circom/iden3 shape).

## Dropped (non-parametrizable with existing infrastructure)

- **StarkNet Pedersen** — its curve isn't an arkworks crate and it hardcodes a
  fixed generator table + shift point + 2-field-element layout. It *fits the
  formula* (`Unsigned` + fixed segmentation `[248,4,248,4]` + offset +
  `XCoordinate`) but would require shipping non-arkworks arithmetic and spec
  constants, so it is intentionally excluded.

## Byte-compatibility caveat

`Deterministic` generators are reproducible but are **not** any given spec's
generator set, and arkworks bit/coordinate conventions are used. The instances
are therefore structurally-correct Pedersen hashes, **not** byte-identical to
circomlib/Zcash. Matching a spec exactly means supplying that spec's generator
points (via `Parameters`) plus its bit/output conventions — a per-spec effort the
skeleton localizes rather than eliminates.

## Test

```
cargo test
```

"""Type stubs for pedersenpy (Rust-backed Pedersen hash functions)."""

class BabyJubjubPedersen:
    """Unsigned-window Pedersen hash over the ERC-2494 Baby Jubjub curve
    (arkworks `pedersen::CRH`). `hash` returns the compressed point."""

    def __init__(self, segments: int, bits_per_window: int) -> None: ...
    def hash(self, data: bytes) -> bytes: ...

class JubjubBoweHopwood:
    """Bowe-Hopwood / Zcash-Sapling Pedersen hash over Jubjub
    (arkworks `bowe_hopwood::CRH`). `hash` returns the x-coordinate."""

    def __init__(self, segments: int, chunks_per_segment: int) -> None: ...
    def hash(self, data: bytes) -> bytes: ...

class CircomPedersen:
    """circom / iden3-compatible Baby Jubjub Pedersen hash (`circomlibjs pedersenHash`).
    Reusable: generators are derived once and cached across calls."""

    def __init__(self) -> None: ...
    def hash(self, data: bytes) -> bytes:
        """Hash `data`, returning the 32-byte packed point (circomlibjs `packPoint`)."""
        ...

class ZcashPedersen:
    """Zcash Sapling Pedersen hash over Jubjub. Reusable across calls."""

    def __init__(self, personalization: bytes | None = ...) -> None: ...
    def hash(self, data: bytes) -> bytes:
        """Hash `data`, returning the 32-byte little-endian u-coordinate."""
        ...

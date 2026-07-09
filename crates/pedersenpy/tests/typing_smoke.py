"""Type-checked usage of the pedersenpy public API.

CI runs `mypy` on this file against the installed wheel, so a stub that drifts
from the exercised surface (missing class, changed signature, wrong return type)
fails the build. Not packaged into the wheel.
"""

import pedersenpy


def main() -> None:
    a = pedersenpy.BabyJubjubPedersen(64, 4)
    d1: bytes = a.hash(b"x")

    b = pedersenpy.JubjubBoweHopwood(16, 40)
    d2: bytes = b.hash(b"x")

    c = pedersenpy.CircomPedersen()
    d3: bytes = c.hash(b"Hello")

    z_default = pedersenpy.ZcashPedersen()
    z_custom = pedersenpy.ZcashPedersen(b"Zcash_PH")
    d4: bytes = z_default.hash(b"Hello")
    d5: bytes = z_custom.hash(b"Hello")

    print(d1.hex(), d2.hex(), d3.hex(), d4.hex(), d5.hex())


if __name__ == "__main__":
    main()

"""Reference-vector and behavior tests for the pedersenpy bindings.

Run against the installed wheel (the artifact users consume), these pin the
*binding's* byte output — FFI marshalling, output serialization, constructor
handling, memoized reuse — which the Rust tests (comparing field elements/points
directly) don't exercise. Vectors are the same ground truth: circomlibjs and
zcash/zcash-test-vectors.
"""

import pedersenpy
import pytest


def seq(n: int) -> bytes:
    return bytes(range(n))


# circom / circomlibjs — 32-byte packed point (hex).
CIRCOM = [
    (b"Hello", "0e90d7d613ab8b5ea7f4f8bc537db6bb0fa2e5e97bbac1c1f609ef9e6a35fd8b"),
    (seq(64), "58b7b97eb2fd6adb8e43a6ec24ee3c27a92bae7e6375a86f426d076d4b3ed826"),
]

# Zcash Sapling — u-coordinate (decimal), returned as 32-byte little-endian.
ZCASH = [
    (b"Hello", 8754254972755604884333948367738998890971419059392001151429652007230018821080),
    (seq(64), 37515569649653130145499701737487402729855929021425639231448526651152569150619),
]


@pytest.mark.parametrize("msg,expected_hex", CIRCOM)
def test_circom_matches_circomlibjs(msg, expected_hex):
    assert pedersenpy.CircomPedersen().hash(msg).hex() == expected_hex


@pytest.mark.parametrize("msg,expected_u", ZCASH)
def test_zcash_matches_reference(msg, expected_u):
    out = pedersenpy.ZcashPedersen().hash(msg)
    assert len(out) == 32
    assert int.from_bytes(out, "little") == expected_u


def test_zcash_default_personalization_matches_explicit():
    msg = b"payload"
    assert pedersenpy.ZcashPedersen().hash(msg) == pedersenpy.ZcashPedersen(b"Zcash_PH").hash(msg)


def test_zcash_custom_personalization_differs():
    msg = b"payload"
    assert pedersenpy.ZcashPedersen(b"Zcash_PH").hash(msg) != pedersenpy.ZcashPedersen(b"OtherPH_").hash(msg)


def test_zcash_personalization_must_be_8_bytes():
    with pytest.raises(ValueError):
        pedersenpy.ZcashPedersen(b"short")


def test_reuse_is_stable_across_growth():
    c = pedersenpy.CircomPedersen()
    first = c.hash(b"Hello")
    _ = c.hash(seq(200))  # grows the memoized generator cache to 8 segments
    assert c.hash(b"Hello") == first  # earlier results unaffected


@pytest.mark.parametrize(
    "ctor",
    [
        lambda: pedersenpy.BabyJubjubPedersen(64, 4),
        lambda: pedersenpy.JubjubBoweHopwood(16, 40),
    ],
)
def test_deterministic_instances(ctor):
    a, b = ctor(), ctor()
    assert a.hash(b"alpha") == b.hash(b"alpha")  # reproducible generators
    assert a.hash(b"alpha") != a.hash(b"omega")  # input-sensitive
    assert len(a.hash(b"alpha")) == 32  # binding returns 32 bytes

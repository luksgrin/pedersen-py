import pedersenpy

def test_babyjubjub_string_number_construction():
    point = pedersenpy.PyBabyjubjubPoint("1", "2")
    assert point.x == 1
    assert point.y == 2

def test_babyjubjub_hex_number_construction():
    point = pedersenpy.PyBabyjubjubPoint("0x1", "0x2")
    assert point.x == 1
    assert point.y == 2

def test_babyjubjub_int_number_construction():
    point = pedersenpy.PyBabyjubjubPoint(1, 2)
    assert point.x == 1
    assert point.y == 2

def test_babyjubjub_bytes_number_construction():
    point = pedersenpy.PyBabyjubjubPoint(b"\x01", b"\x02")
    assert point.x == 1
    assert point.y == 2

def test_babyjubjub_dict():
    point = pedersenpy.PyBabyjubjubPoint(1, 2)
    assert point.__dict__ == {"x": 1, "y": 2}

def test_babyjubjub_mul_scalar():
    point = pedersenpy.PyBabyjubjubPoint(1, 2).mul_scalar(3)
    assert point.x == 3797457032829818846051130920114956979086572769243247264622180348175076696534
    assert point.y == 2021268640777692167800740213637109364295083033904614886339895090764087138703

def test_babyjubjub_native_mul_scalar():
    point = pedersenpy.PyBabyjubjubPoint(1, 2) * 3
    assert point.x == 3797457032829818846051130920114956979086572769243247264622180348175076696534
    assert point.y == 2021268640777692167800740213637109364295083033904614886339895090764087138703

def test_babyjubjub_eq():
    point1 = pedersenpy.PyBabyjubjubPoint(1, 2) * 3
    point2 = pedersenpy.PyBabyjubjubPoint(
        "0x086548d5d4d8f82f6a6baf21894b5e46607423daf13e18c2d2634b3cc84e21d6",
        "0x0477ff5cbee2683992acc7fb0a4806293e9c9d65eedd650315e360e98851998f"
    )
    assert point1 == point2

def test_babyjubjub_ne():
    point1 = pedersenpy.PyBabyjubjubPoint(1, 2)
    point2 = pedersenpy.PyBabyjubjubPoint(2, 3)
    assert point1 != point2

def test_babyjubjub_compress():
    point = pedersenpy.PyBabyjubjubPoint(1, 2)
    assert point.compress() == b"\x02\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00"

def test_babyjubjub_compress_hex():
    point = pedersenpy.PyBabyjubjubPoint(1, 2)
    assert point.compress_hex() == "0x0200000000000000000000000000000000000000000000000000000000000000"

def test_babyjubjub_projective():
    point = pedersenpy.PyBabyjubjubPoint(1, 2).to_projective()
    assert point.x == 1
    assert point.y == 2
    assert point.z == 1

def test_babyjubjub_projective_to_affine():
    point = pedersenpy.PyBabyjubjubPoint(1, 2).to_projective().to_affine()
    assert point.x == 1
    assert point.y == 2

def test_babyjubjub_projective_construction():
    point1 = pedersenpy.PyBabyjubjubPointProjective(1, 2, 1)
    point2 = pedersenpy.PyBabyjubjubPoint(1, 2).to_projective()
    assert point1 == point2

def test_babyjubjub_projective_mul_scalar():
    point = pedersenpy.PyBabyjubjubPointProjective(1, 2, 1).mul_scalar(3)
    assert point.x == 3797457032829818846051130920114956979086572769243247264622180348175076696534
    assert point.y == 2021268640777692167800740213637109364295083033904614886339895090764087138703
    assert point.z == 1
import pedersenpy

def test_babyjubjub_string_number_construction():
    point = pedersenpy.PyBabyjubjubPoint("1", "2")
    assert point.x() == 1
    assert point.y() == 2

def test_babyjubjub_hex_number_construction():
    point = pedersenpy.PyBabyjubjubPoint("0x1", "0x2")
    assert point.x() == 1
    assert point.y() == 2

def test_babyjubjub_int_number_construction():
    point = pedersenpy.PyBabyjubjubPoint(1, 2)
    assert point.x() == 1
    assert point.y() == 2
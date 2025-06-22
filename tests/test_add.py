import pedersenpy

def test_add_pass():
    assert pedersenpy.add(1, 2) == 3

def test_add_fail():
    assert pedersenpy.add(1, 2) != 4
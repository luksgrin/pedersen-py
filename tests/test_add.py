import pedersen_py_backend

def test_add_pass():
    assert pedersen_py_backend.add(1, 2) == 3

def test_add_fail():
    assert pedersen_py_backend.add(1, 2) != 4
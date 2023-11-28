from importlib import import_module

KYBER_VERSION = "768"


def get_security_level():
    return KYBER_VERSION


def set_security_level(ver):
    global KYBER_VERSION
    if type(ver) is int:
        ver = str(ver)
    assert ver in ["512", "768", "1024"]
    KYBER_VERSION = ver
    print(f"Set Kyber security level to {KYBER_VERSION}")


def get_imports():
    KYBER_VERSION = get_security_level()
    check_bp = import_module(f"check_bp{KYBER_VERSION}")
    python_kyber = import_module(f"python_kyber{KYBER_VERSION}")
    return KYBER_VERSION, check_bp, python_kyber

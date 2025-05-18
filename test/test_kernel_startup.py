import pytest
from conftest import QEMUConnection


@pytest.mark.dependency()
def test_kernel_boot(qemu: QEMUConnection):
    """Test that kernel boots and outputs expected startup messages"""
    # Check for kernel entry message
    output = qemu.read_until(b"Entering JOS Kernel")
    assert b"Entering JOS Kernel" in output

    # Check for initialization messages
    messages = [
        b"Initialized High Precision Event Timer",
        b"Initialized Serial Port",
        b"Initialized Kernel Heap Memory",
        b"Initialized Global Descriptor Table",
        b"Initialized Interrupt Descriptor Table",
        b"Initialized Filesystem",
    ]

    for message in messages:
        output = qemu.read_until(message)
        assert message in output

    # Check for final boot message
    output = qemu.read_until(b"JOS Kernel initialized; switching to userland")
    assert b"JOS Kernel initialized; switching to userland" in output


@pytest.mark.dependency(depends=["test_kernel_boot"])
@pytest.mark.dependency()
def test_userland(qemu: QEMUConnection):
    """Test that userland starts and outputs expected messages"""

    output = qemu.read_until(b"Hallo Carina")
    assert b"Hallo Carina" in output


@pytest.mark.dependency(depends=["test_userland"])
@pytest.mark.dependency()
def test_userland_doom(qemu: QEMUConnection):
    """Test that userland Doom starts and outputs expected messages"""
    # Check for Doom initialization messages
    messages = [
        b"DOOM Shareware Startup",
        b"V_Init: allocate screens.",
        b"M_LoadDefaults: Load system defaults.",
        b"Z_Init: Init zone memory allocation daemon.",
        b"W_Init: Init WADfiles.",
        b"Shareware!",
        b"M_Init: Init miscellaneous info.",
        b"R_Init: Init DOOM refresh daemon -",
        b"P_Init: Init Playloop state.",
        b"I_Init: Setting up machine state.",
        b"D_CheckNetGame: Checking network game status.",
        b"S_Init: Setting up sound.",
        b"HU_Init: Setting up heads up display.",
        b"ST_Init: Init status bar.",
    ]

    for message in messages:
        output = qemu.read_until(message)
        assert message in output


@pytest.mark.dependency(depends=["test_userland_doom"])
def test_retrieve_profiling(qemu: QEMUConnection):
    # send button press to qemu
    qemu.send_key_press("l")

    output = qemu.read_until(b"Tracepoints logged")
    assert b"Tracepoints logged" in output

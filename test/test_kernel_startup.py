import pytest
from conftest import QEMUConnection


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

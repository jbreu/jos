import pytest
import socket
import time
from typing import Generator
import subprocess
import sys
import os


class QEMUConnection:
    def __init__(self, host: str = "127.0.0.1", port: int = 4444):
        self.process = None
        self.log_file = None

        # Clear serial.log before starting
        try:
            with open("serial.log", "w") as f:
                f.truncate(0)
        except Exception as e:
            print(f"Warning: Could not clear serial.log: {e}")

        qemu_command = "qemu-system-x86_64"
        # Check if running on Windows or WSL, as both might need .exe
        is_windows = sys.platform == "win32"
        is_wsl = False
        if sys.platform == "linux":
            try:
                # A common way to detect WSL is checking /proc/version
                with open("/proc/version", "r") as f:
                    content = f.read().lower()
                    if "microsoft" in content or "wsl" in content:
                        is_wsl = True
            except FileNotFoundError:
                pass  # /proc/version doesn't exist, likely not WSL

        if is_windows or is_wsl:
            qemu_command += ".exe"

        # Check if current directory is test
        current_dir = os.path.basename(os.getcwd())
        iso_path = (
            "../dist/x86_64/kernel.iso"
            if current_dir == "test"
            else "dist/x86_64/kernel.iso"
        )

        command = [
            qemu_command,
            "-nographic",  # Add this option to run in headless mode
            "-action",
            "panic=pause",
            "-no-reboot",
            "-serial",
            "tcp:127.0.0.1:4444,server,nowait",
            "-monitor",
            "stdio",
            "-cdrom",
            iso_path,
        ]
        try:
            # Ensure the log file is opened before starting the process
            self.log_file = open("qemu.log", "wb")
            self.process = subprocess.Popen(
                command, stdout=self.log_file, stderr=self.log_file
            )
            # A small delay might be needed for QEMU to start the serial server
            # Adjust the sleep duration if connection issues persist
            time.sleep(1.0)
        except FileNotFoundError:
            print(
                "Error: qemu-system-x86_64 command not found. Is QEMU installed and in PATH?"
            )
            if self.log_file:
                self.log_file.close()  # Close the file if opened before error
            raise
        except Exception as e:
            print(f"Failed to start QEMU: {e}")
            if self.log_file:
                self.log_file.close()  # Close the file if opened before error
            raise
        self.host = host
        self.port = port
        self.socket = None

    def connect(self, retries: int = 5, delay: float = 1.0) -> None:
        """Connect to QEMU's serial console with retries"""
        for attempt in range(retries):
            try:
                self.socket = socket.create_connection((self.host, self.port))
                return
            except (ConnectionRefusedError, socket.error):
                if attempt < retries - 1:
                    time.sleep(delay)
                    continue
                raise

    def disconnect(self) -> None:
        """Close the connection"""
        if self.socket:
            self.socket.close()
            self.socket = None

        if self.process:
            try:
                self.process.terminate()
                self.process.wait(timeout=5)  # Wait for the process to terminate
            except subprocess.TimeoutExpired:
                self.process.kill()  # Force kill if terminate doesn't work
            except Exception as e:
                print(f"Error terminating QEMU process: {e}")
            finally:
                self.process = None

    def read_until(self, marker: bytes, timeout: float = 30.0) -> bytes:
        """Read from socket until marker is found or timeout occurs"""
        self.socket.settimeout(timeout)
        data = b""
        start_time = time.time()

        # Open serial.log in append binary mode
        with open("serial.log", "ab") as log_file:
            while True:
                if time.time() - start_time > timeout:
                    raise TimeoutError(f"Timeout waiting for marker {marker}")

                try:
                    chunk = self.socket.recv(1024)
                    if not chunk:
                        raise ConnectionError("Connection closed by remote host")
                    # Write received chunk to log file
                    log_file.write(chunk)
                    log_file.flush()  # Ensure data is written immediately
                    data += chunk
                    if marker in data:
                        return data
                except socket.timeout:
                    raise TimeoutError(f"Timeout waiting for marker {marker}")


@pytest.fixture
def qemu() -> Generator[QEMUConnection, None, None]:
    """Fixture that provides a QEMU serial connection"""
    connection = QEMUConnection()
    connection.connect()
    yield connection
    connection.disconnect()

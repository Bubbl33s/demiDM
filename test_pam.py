#!/usr/bin/env python3
import sys
import os

try:
    import pam
except ImportError:
    print("python-pam not installed. Install with: pip install python-pam")
    sys.exit(1)

p = pam.pam()
username = sys.argv[1] if len(sys.argv) > 1 else "bubbles"
password = sys.argv[2] if len(sys.argv) > 2 else "test"

result = p.authenticate(username, password, service='demidm')
print(f"Authentication result: {result}")
print(f"Reason: {p.reason}")
print(f"Code: {p.code}")

#!/usr/bin/env python3
import os

LICENSE_LINE = (
    "// SPDX-License-Identifier: MIT OR Apache-2.0\n\n"  # Change to your license
)

for root, _, files in os.walk("."):
    for file in files:
        if file.endswith(".rs"):
            path = os.path.join(root, file)
            with open(path, "r+", encoding="utf-8") as f:
                content = f.read()
                if LICENSE_LINE.strip() not in content.splitlines()[0]:
                    f.seek(0, 0)
                    f.write(LICENSE_LINE + content)
                    print(f"Added SPDX to {path}")

#!/bin/bash
# Test parameter expansion with space before negative offset

TEXT=hello_world
echo "Test: ${TEXT: -5}"
echo "Expected: world"

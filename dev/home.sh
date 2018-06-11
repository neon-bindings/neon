#!/bin/bash

# home.sh: Print the complete path to the Neon project root directory.

(cd $(dirname $0)/.. && pwd)

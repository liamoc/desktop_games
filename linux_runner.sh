#!/bin/sh

export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:./lib64/
./`basename $0`.x86_64

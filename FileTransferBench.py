#!/usr/bin/env python
# -*- coding: utf-8 -*-

import shutil
import os
import time

foo = "C:/Users/berna/Desktop/Test Folder/SOURCE/Cannonade.flac"
bar = "C:/Users/berna/Desktop/Test Folder/TARGET1/"
noc = "C:/Users/berna/Desktop/Test Folder/TARGET2/"


# Source file location, target directory, whether to keep the filename intact
# and whether to create the target directory in case it doesn't exist.
def CopyFile(SrcFile, TgtDir, KeepName=True, MakeDir=True):
    fsrc = None
    fdst = None
    keepGoing = False
    # Checks is TgtDir is valid and creates if needed.
    if MakeDir and not os.path.isdir(TgtDir):
        os.makedirs(TgtDir)
    # Processes TgtDir depending on filename choice.
    if KeepName is True:
        TgtDir += os.path.basename(SrcFile)
        print(TgtDir)
    try:
        fsrc = open(SrcFile, 'rb')
        fdst = open(TgtDir, 'wb')
        keepGoing = True
        count = 0
        while keepGoing:
            # Read blocks of size 2**20 = 1048576
            buf = fsrc.read(2 ** 20)
            if not buf:
                break
            fdst.write(buf)
            count += len(buf)
    finally:
        if fdst:
            fdst.close()
        if fsrc:
            fsrc.close()
    return keepGoing

start_time = time.time()
CopyFile(foo, bar)
TimeOne = time.time() - start_time
print("Method one---> %s seconds" % (TimeOne))
start_time = time.time()
shutil.copy(foo, noc)
TimeTwo = time.time() - start_time
print("Method two---> %s seconds" % (TimeTwo))
print("Time delta---> {}".format(TimeOne-TimeTwo))

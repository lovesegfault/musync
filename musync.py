#!/usr/bin/env python
# -*- coding: utf-8 -*-
import shutil
import argparse
import os
from mutagen.flac import FLAC
from os import listdir
from fuzzywuzzy import fuzz

ntag = 0
full = 0
nvan = 0
nap = 0


def metaprint(meta):
    for tag in meta:
        print("%s: %s" % (tag, meta[tag]))


def comptag(tgtf, srcf):
    # print srcf
    if (not os.path.isfile(tgtf)):
        files = listdir(os.path.dirname(tgtf))
        diffs = []
        for song in files:
            diffs.append(fuzz.ratio(song, os.path.basename(srcf)))
        bmatch = os.path.dirname(tgtf) + '/' + files[diffs.index(max(diffs))]
        os.rename(bmatch, os.path.dirname(tgtf) + '/' + os.path.basename(srcf))
    tagtgt = FLAC(tgtf)
    tagsrc = FLAC(srcf)
    if (set(tagtgt) != set(tagsrc)):
        newtags = list(set(tagtgt) - set(tagsrc))
        for tag in newtags:
            tagsrc["%s" % tag] = tagtgt[tag]
            tagsrc.save(srcf)
    for tag in tagtgt:
        if tagtgt[tag] != tagsrc[tag]:
            ntag += 1
            print(
                "There's a difference! The %s field doesn't match." % tag)
            tagsrc[tag] = tagtgt[tag]
            tagsrc.save(srcf)
            metaprint(tagsrc)
    print("Done comparing tags.")

parser = argparse.ArgumentParser(description='Manages music libraries')
parser.add_argument(
    '-full', help='Runs a full scan automatically', action='store_true')
args = parser.parse_args()
if args.full:
    full = 1
dev = '/home/bernardo/Musync/F2/'  # Your device.
lib = '/home/bernardo/Musync/F1/'  # Your music folder.
R1 = listdir(dev)
R2 = listdir(lib)
vanished = [filename for filename in R1 if filename not in R2]
appeared = [filename for filename in R2 if filename not in R1]
if vanished:
    for folder in vanished:
        shutil.rmtree(dev + folder)
        nvan += 1
        print("Deprecated folders successfuly deleted.")
else:
    print("No folders to delete from the device.")
if appeared:
    for folder in appeared:
        shutil.copytree(lib + folder, dev + folder)
        nap += 1
        print("New folders successfuly moved.")
else:
    print("No new folders to move.")
inopt = input("Would you like to do a filechange scan? (Y/n)")
if (inopt == 'y' or inopt == 'Y' or inopt == '1' or full):
    for artist in R2:
        for album in listdir(lib + artist):
            for song in listdir(lib + artist + '/' + album):
                comptag(dev + artist + '/' + album + '/' + song,
                        lib + artist + '/' + album + '/' + song)
print('Sync complete. %d items deleted, %d items moved and %d files changed' %
      (nvan, nap, ntag))

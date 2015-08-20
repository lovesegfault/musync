#!/usr/bin/env python
# -*- coding: utf-8 -*-
import shutil
import os

from mutagen.flac import FLAC  # Used for metadata handling.
from os import listdir  # Used for general operations.
from fuzzywuzzy import fuzz  # Last resource name association.
# Insert here the root directory of your library and device respectively.
lib = 'C:/Users/berna/Desktop/Lib/'
dev = 'C:/Users/berna/Desktop/Dev/'


# Faster file copying function, arguments go as follows: Source file location,
# target directory, whether to keep the filename intact and whether to create
# the target directory in case it doesn't exist.
def copy_file(SrcFile, TgtDir, KeepName=True):
    SourceFile = None
    TargetFile = None
    KeepGoing = False

    # Processes TgtDir depending on filename choice.
    if KeepName:
        TgtDir += os.path.basename(SrcFile)
        print(TgtDir)
    try:
        SourceFile = open(SrcFile, 'rb')
        TargetFile = open(TgtDir, 'wb')
        KeepGoing = True
        Count = 0
        while KeepGoing:
            # Read blocks of size 2**20 = 1048576
            Buffer = SourceFile.read(2 ** 20)
            if not Buffer:
                break
            TargetFile.write(Buffer)
            Count += len(Buffer)
    finally:
        if TargetFile:
            TargetFile.close()
        if SourceFile:
            SourceFile.close()
    return KeepGoing


# XXX Ugly workaround-ish, needs improvement.
# Copies a directory (SrcDir) to TgtDir
def copy_tree(SrcDir, TgtDir, Replace=True, Repeated=False):
    # Checks if function is being used for self-recursiveness.
    if not Repeated:
        # If not handles folder naming (check function call to understand this
        # is a weird workaround that just happened to work and I ain't touching
        # it no more.
        TgtDir = format_dir(TgtDir, os.path.basename(SrcDir.rstrip("/")))
    if not os.path.isdir(TgtDir):
        os.makedirs(TgtDir)
    # Makes Subs as all files and folders in the folder to be copied.
    Subs = listdir(SrcDir)
    Errors = []
    # For every subdirectory/file in Source
    for Sub in Subs:
        # Process filenames to remove last "/" and pick Src/Sub and Tgt/Sub
        SrcName = format_dir(SrcDir, Sub).rstrip("/")
        TgtName = format_dir(TgtDir, Sub).rstrip("/")
        # Using try because file operatins are nasty business.
        try:
            # If it's a dir inside a dir we use this very function to copy it,
            # recursiveness at it's best. Can cause a whole bunch of weird
            # behaviour, don't mess with it too much.
            if os.path.isdir(SrcName):
                copy_tree(SrcName, TgtName, Repeated=True)
            # If it's just a file call copy_file and that's it, KeepName is set
            # as false because it's function is being done by the very logic
            # of this for loop.
            else:
                copy_file(SrcName, TgtName, KeepName=False)
        # If things get kinky grab the error and report.
        except (IOError, os.error) as why:
            Errors.append((SrcName, TgtName, str(why)))
        if Errors:
            raise Exception(Errors)


# Checks for new and deleted folders and returns their name.
def check_folder(SrcDir, TgtDir):
    # Lists Source and Target folder.
    Source = listdir(SrcDir)
    Target = listdir(TgtDir)
    # Then creates a list of deprecated and new directories.
    Deleted = [FileName for FileName in Target if FileName not in Source]
    Added = [FileName for FileName in Source if FileName not in Target]
    # Returns both lists.
    return (Added, Deleted)


# Checks for song in case there's a name mismatch or missing file.
def check_song(SrcFile, TgtDir):
    Matches = []
    # Invariably the new name will be that of the source file, the issue here
    # is finding which song is the correct one.
    NewName = TgtDir + '/' + os.path.basename(SrcFile)
    TagSource = FLAC(SrcFile)
    # Grabs the number of samples in the original file.
    SourceSamples = TagSource.info.total_samples
    # Checks if any song has a matching sample number and if true appends the
    # song's filename to Matches[]
    for Song in listdir(TgtDir):
        SongInfo = FLAC(TgtDir + '/' + Song)
        if (SongInfo.info.total_samples == SourceSamples):
            Matches.append(Song)
    # If two songs have the same sample rate (44100Hz for CDs) and the same
    # length it matches them to the source by filename similarity.
    if (Matches.count > 1):
        Diffs = []
        for Song in Matches:
            Diffs.append(fuzz.ratio(Song, os.path.basename(SrcFile)))
        if (max(Diffs) > 0.8):
            BestMatch = TgtDir + '/' + Matches[Diffs.index(max(Diffs))]
            os.rename(BestMatch, NewName)
        else:
            shutil.copy(SrcFile, TgtDir)
    # If there's no match at all simply copy over the missing file.
    elif (Matches.count == 0):
        shutil.copy(SrcFile, TgtDir)
    # If a single match is found the filename will be the first item on the
    # Matches[] list.
    else:
        os.rename(TgtDir + '/' + Matches[0], NewName)


# Syncs folders in a directory and return the change count.
def sync(SrcDir, TgtDir):
    AddCount = 0
    DeleteCount = 0
    # Grabs the folders to be added and deleted.
    NewDir, OldDir = check_folder(SrcDir, TgtDir)
    # Checks if any and then does add/rm.
    if OldDir:
        for Folder in OldDir:
            shutil.rmtree(TgtDir + Folder)
            DeleteCount += 1
    if NewDir:
        for Folder in NewDir:
            copy_tree(format_dir(SrcDir, Folder), TgtDir)
            AddCount += 1
    return(AddCount, DeleteCount)


# Fixes missing metadata fields.
def fix_metadata(SrcFile, TgtFile):
    TagSource = FLAC(TgtFile)
    TagTarget = FLAC(SrcFile)
    # Checks for deleted tags on source file and deletes them from target.
    if (set(TagTarget) - set(TagSource)):
        OldTags = list(set(TagTarget) - set(TagSource))
        for Tag in OldTags:
            # TODO Right now I haven't quite figured out how to delete
            # specific tags, so workaround is to delete them all.
            TagTarget.delete()
    # Checks for new tags on source file and transfers them to target.
    if (set(TagSource) != set(TagTarget)):
        NewTags = list(set(TagSource) - set(TagTarget))
        for Tag in NewTags:
            TagTarget["%s" % Tag] = TagSource[Tag]
            TagTarget.save(TgtFile)


# Does metadata transfer between two files.
def match_metadata(SrcFile, TgtFile):
    Altered = 0
    TagSource = FLAC(SrcFile)
    TagTarget = FLAC(TgtFile)
    # For every different Tag in source song copy it to target and save.
    for Tag in TagSource:
        if TagSource[Tag] != TagTarget[Tag]:
            Altered += 1
            TagTarget[Tag] = TagSource[Tag]
            TagTarget.save(TgtFile)
    return(Altered)


# Simply does directory formatting to make things easier.
def format_dir(Main, Second, Third=""):
    # Replaces \ with /
    Main = Main.replace('\\', '/')
    # Adds a / to the end of Main and concatenates Main and Second.
    if(Main[len(Main) - 1] != '/'):
        Main += '/'
    Main += Second + '/'
    # Concatenates Main and Third if necessary.
    if (Third):
        Main += Third + '/'
    return (Main)

# Sync main folders in lib with dev.
sync(lib, dev)
# For every Artist in lib sync it's Albums
for Artist in listdir(lib):
    sync(format_dir(lib, Artist), format_dir(dev, Artist))
    # For every Album in Artist match songs
    for Album in listdir(format_dir(lib, Artist)):
        # Declares lib Album and dev Album to make function calls shorter.
        CurrentAlbum = format_dir(lib, Artist, Album)
        CoAlbum = format_dir(dev, Artist, Album)
        for Song in listdir(CurrentAlbum):
            if (".flac" in Song or ".FLAC" in Song):
                try:
                    # Tries to match lib and dev song's metadata.
                    match_metadata(CurrentAlbum + Song, CoAlbum + Song)
                except:
                    # If that fails will try to fix both Filename and Tag
                    # fields.
                    check_song(CurrentAlbum + Song, CoAlbum)
                    fix_metadata(CurrentAlbum + Song, CoAlbum + Song)
                    try:
                        # Try again after fix.
                        match_metadata(CurrentAlbum + Song, CoAlbum + Song)
                    except Exception as e:
                        # If it still doesn't work there's black magic in place
                        # go sleep, drink a beer and try again later.
                        print("""Ehm, something happened and your sync failed.\n
                              Error:{}""".format(e))
                        raise SystemExit(0)

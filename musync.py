#!/usr/bin/env python
# -*- coding: utf-8 -*-
import shutil
import os

from mutagen.flac import FLAC  # Used for metadata handling.
from os import listdir  # Used for general operations.
from fuzzywuzzy import fuzz  # Last resource name association.
# Insert here the root directory of your library and device respectively.
lib = ''
dev = ''


# Checks for new and deleted folders and returns their name.
def check_folder(SrcDir, TgtDir):
    # Lists Source and Target folder.
    Source = listdir(SrcDir)
    Target = listdir(TgtDir)
    # Then creates a list of deprecated and new directories.
    Deleted = [Filename for Filename in Target if Filename not in Source]
    Added = [FileName for FileName in Source if Filename not in Target]
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
        BestMatch = TgtDir + '/' + Matches[Diffs.index(max(Diffs))]
        os.rename(BestMatch, NewName)
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
            shutil.copytree(SrcDir + Folder, TgtDir + Folder)
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


def match_metadata(SrcFile, TgtFile):
    # TODO: Will do the metadata transfer between two files.
    return()

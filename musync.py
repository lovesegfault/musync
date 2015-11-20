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


# Faster file copying function, arguments go as follows: source file location,
# target directory, whether to keep the file_name intact and whether to create
# the target directory in case it doesn't exist.
def copy_file(source_file, target_dir, keep_name=True):
    source_file = None
    target_file = None
    keep_going = False

    # Processes target_dir depending on file_name choice.
    if keep_name:
        target_dir += os.path.basename(source_file)
        print(target_dir)
    try:
        source_file = open(source_file, 'rb')
        target_file = open(target_dir, 'wb')
        keep_going = True
        Count = 0
        while keep_going:
            # Read blocks of size 2**20 = 1048576
            file_buffer = source_file.read(2 ** 20)
            if not file_buffer:
                break
            target_file.write(file_buffer)
            Count += len(file_buffer)
    finally:
        if target_file:
            target_file.close()
        if source_file:
            source_file.close()
    return keep_going


# XXX Ugly workaround-ish, needs improvement.
# Copies a directory (source_dir) to target_dir
def copy_tree(source_dir, target_dir, Replace=True, repeated=False):
    # Checks if function is being used for self-recursiveness.
    if not repeated:
        # If not handles folder naming (check function call to understand this
        # is a weird workaround that just happened to work and I ain't touching
        # it no more.
        target_dir = format_dir(
            target_dir, os.path.basename(source_dir.rstrip("/")))
    if not os.path.isdir(target_dir):
        os.makedirs(target_dir)
    # Makes subdirs as all files and folders in the folder to be copied.
    subdirs = listdir(source_dir)
    errors = []
    # For every subdirectory/file in source
    for Sub in subdirs:
        # Process file_names to remove last "/" and pick Src/Sub and Tgt/Sub
        source_name = format_dir(source_dir, Sub).rstrip("/")
        target_name = format_dir(target_dir, Sub).rstrip("/")
        # Using try because file operatins are nasty business.
        try:
            # If it's a dir inside a dir we use this very function to copy it,
            # recursiveness at it's best. Can cause a whole bunch of weird
            # behaviour, don't mess with it too much.
            if os.path.isdir(source_name):
                copy_tree(source_name, target_name, repeated=True)
            # If it's just a file call copy_file and that's it, keep_name is set
            # as false because it's function is being done by the very logic
            # of this for loop.
            else:
                copy_file(source_name, target_name, keep_name=False)
        # If things get kinky grab the error and report.
        except (IOError, os.error) as why:
            errors.append((source_name, target_name, str(why)))
        if errors:
            raise Exception(errors)


# Checks for new and gone folders and returns their name.
def check_folder(source_dir, target_dir):
    # Lists source and target folder.
    source = listdir(source_dir)
    target = listdir(target_dir)
    # Then creates a list of deprecated and new directories.
    gone = [file_name for file_name in target if file_name not in source]
    new = [file_name for file_name in source if file_name not in target]
    # Returns both lists.
    return (new, gone)


# Checks for song in case there's a name mismatch or missing file.
def check_song(source_file, target_dir):
    matches = []
    # Invariably the new name will be that of the source file, the issue here
    # is finding which song is the correct one.
    new_name = target_dir + '/' + os.path.basename(source_file)
    tag_source = FLAC(source_file)
    # Grabs the number of samples in the original file.
    source_samples = tag_source.info.total_samples
    # Checks if any song has a matching sample number and if true appends the
    # song's file_name to matches[]
    for music_file in listdir(target_dir):
        SongInfo = FLAC(target_dir + '/' + music_file)
        if (SongInfo.info.total_samples == source_samples):
            matches.append(music_file)
    # If two songs have the same sample rate (44100Hz for CDs) and the same
    # length it matches them to the source by file_name similarity.
    if (matches.count > 1):
        deltas = []
        for music_file in matches:
            deltas.append(fuzz.ratio(
                music_file, os.path.basename(source_file)))
        if (max(deltas) > 0.8):
            top_match = target_dir + '/' + matches[deltas.index(max(deltas))]
            os.rename(top_match, new_name)
        else:
            shutil.copy(source_file, target_dir)
    # If there's no match at all simply copy over the missing file.
    elif (matches.count == 0):
        shutil.copy(source_file, target_dir)
    # If a single match is found the file_name will be the first item on the
    # matches[] list.
    else:
        os.rename(target_dir + '/' + matches[0], new_name)


# Syncs folders in a directory and return the change count.
def sync(source_dir, target_dir):
    add_count = 0
    del_count = 0
    # Grabs the folders to be new and gone.
    new_dir, old_dir = check_folder(source_dir, target_dir)
    # Checks if any and then does add/rm.
    if old_dir:
        for directory in old_dir:
            shutil.rmtree(target_dir + directory)
            del_count += 1
    if new_dir:
        for directory in new_dir:
            copy_tree(format_dir(source_dir, directory), target_dir)
            add_count += 1
    return(add_count, del_count)


# Fixes missing metadata fields.
def fix_metadata(source_file, target_file):
    tag_source = FLAC(target_file)
    tag_target = FLAC(source_file)
    # Checks for gone tags on source file and deletes them from target.
    if (set(tag_target) - set(tag_source)):
        OldTags = list(set(tag_target) - set(tag_source))
        for Tag in OldTags:
            # TODO Right now I haven't quite figured out how to delete
            # specific tags, so workaround is to delete them all.
            tag_target.delete()
    # Checks for new tags on source file and transfers them to target.
    if (set(tag_source) != set(tag_target)):
        NewTags = list(set(tag_source) - set(tag_target))
        for Tag in NewTags:
            tag_target["%s" % Tag] = tag_source[Tag]
            tag_target.save(target_file)


# Does metadata transfer between two files.
def match_metadata(source_file, target_file):
    change_count = 0
    tag_source = FLAC(source_file)
    tag_target = FLAC(target_file)
    # For every different Tag in source song copy it to target and save.
    for Tag in tag_source:
        if tag_source[Tag] != tag_target[Tag]:
            change_count += 1
            tag_target[Tag] = tag_source[Tag]
            tag_target.save(target_file)
    return(change_count)


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
# For every artist in lib sync it's Albums
for artist in listdir(lib):
    sync(format_dir(lib, artist), format_dir(dev, artist))
    # For every album in artist match songs
    for album in listdir(format_dir(lib, artist)):
        # Declares lib album and dev album to make function calls shorter.
        current_album = format_dir(lib, artist, album)
        CoAlbum = format_dir(dev, artist, album)
        for music_file in listdir(current_album):
            if (".flac" in music_file or ".FLAC" in music_file):
                try:
                    # Tries to match lib and dev song's metadata.
                    match_metadata(current_album + music_file,
                                   CoAlbum + music_file)
                except:
                    # If that fails will try to fix both file_name and Tag
                    # fields.
                    check_song(current_album + music_file, CoAlbum)
                    fix_metadata(current_album + music_file,
                                 CoAlbum + music_file)
                    try:
                        # Try again after fix.
                        match_metadata(current_album + music_file,
                                       CoAlbum + music_file)
                    except Exception as e:
                        # If it still doesn't work there's black magic in place
                        # go sleep, drink a beer and try again later.
                        print("""Ehm, something happened and your sync failed.\n
                              Error:{}""".format(e))
                        raise SystemExit(0)

//
// Created by meurer on 20/04/17.
//

#ifndef MUSYNC_LIBSYNC_HPP
#define MUSYNC_LIBSYNC_HPP

#include <boost/foreach.hpp>
#include <boost/filesystem.hpp>

namespace bfs = boost::filesystem;


/**
 * Gets all files and folders in a directory
 * @param directory Directory to scan
 * @param depth Depth of search, starting at 0. A negative depth
 * @return std::vector with boost::filesystem::path's for each object found
 */
std::vector<bfs::path> getObjects(bfs::path directory, int depth) {
    std::vector<bfs::path> object_list;
    bfs::directory_iterator iter(directory), eod;

    BOOST_FOREACH(bfs::path const &path, std::make_pair(iter, eod)) {
                    if (bfs::is_directory(path)) {
                        object_list.push_back(path);
                        if (depth > 0) {
                            std::vector<bfs::path> inner = getObjects(path, depth - 1);
                            object_list.insert(object_list.end(), inner.begin(), inner.end());
                        }
                    } else if (bfs::is_regular_file(path)) {
                        object_list.push_back(path);
                    }
                }
    std::sort(object_list.begin(), object_list.end());
    return object_list;
}

/**
 * Gets all the files in a directory
 * @param directory Directory to scan
 * @return std::vector with boost::filesystem::path of each file
 */
std::vector<bfs::path> getFiles(bfs::path directory, int depth) {
    std::vector<bfs::path> file_list;
    bfs::directory_iterator iter(directory), eod;

    BOOST_FOREACH(bfs::path const &path, std::make_pair(iter, eod)) {
                    if (bfs::is_directory(path) && depth > 0) {
                        std::vector<bfs::path> inner = getFiles(path, depth - 1);
                        file_list.insert(file_list.end(), inner.begin(), inner.end());
                    } else if (bfs::is_regular_file(path)) {
                        file_list.push_back(path);
                    }
                }
    std::sort(file_list.begin(), file_list.end());
    return file_list;
}

std::vector<bfs::path> getDirectories(bfs::path directory, int depth) {
    std::vector<bfs::path> dir_list;
    bfs::directory_iterator iter(directory), eod;

    BOOST_FOREACH(bfs::path const &path, std::make_pair(iter, eod)) {
                    if (bfs::is_directory(path)) {
                        dir_list.push_back(path);
                        if (depth > 0) {
                            std::vector<bfs::path> inner = getDirectories(path, depth - 1);
                            dir_list.insert(dir_list.end(), inner.begin(), inner.end());
                        }
                    }
                }
    std::sort(dir_list.begin(), dir_list.end());
    return dir_list;
}

#endif //MUSYNC_LIBSYNC_HPP

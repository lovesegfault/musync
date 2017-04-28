#include <iostream>
#include <boost/filesystem.hpp>
#include "libsync.hpp"
namespace bfs = boost::filesystem;

int main() {
    bfs::path source_dir = "/home/meurer/test/a";
    bfs::path target_dir = "/home/meurer/test/b";
    std::vector<bfs::path> list = getObjects(source_dir, 1);
    for(auto i = list.begin(); i != list.end(); ++i){
        std::cout << bfs::canonical(*i).string() << std::endl;
    }
    return 0;
}


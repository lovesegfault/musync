cmake_minimum_required(VERSION 3.7)
project(musync)

find_package(Boost COMPONENTS system filesystem REQUIRED)
INCLUDE_DIRECTORIES(${Boost_INCLUDE_DIR})

set(CMAKE_CXX_STANDARD 11)
set(SOURCE_FILES main.cpp libsync.hpp)

add_library(libsync.a ${SOURCE_FILES})
add_executable(musync ${SOURCE_FILES})

target_link_libraries(musync ${Boost_LIBRARIES} libsync.a)
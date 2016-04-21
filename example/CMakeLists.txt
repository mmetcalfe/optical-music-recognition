cmake_minimum_required(VERSION 3.4)

FIND_PACKAGE(ArrayFire)
INCLUDE_DIRECTORIES(${ArrayFire_INCLUDE_DIRS})

# If you intend to use OpenCL, you need to find it
FIND_PACKAGE(OpenCL)
SET(EXTRA_LIBS ${CMAKE_THREAD_LIBS_INIT} ${OpenCL_LIBRARIES})

ADD_EXECUTABLE(af_orb_test af_orb_test.cpp)
TARGET_LINK_LIBRARIES(af_orb_test ${ArrayFire_LIBRARIES} ${EXTRA_LIBS})

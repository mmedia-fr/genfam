# GenFam — noyau Rust (core/), interface Qt 6 / QML. Socle MMEDIA (cf. MMdedit).
cmake_minimum_required(VERSION 3.24)
project(genfam VERSION 8.0.0 LANGUAGES CXX)

if(CMAKE_CXX_COMPILER_ID STREQUAL "MSVC")
  set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreadedDLL")
endif()

set(CMAKE_AUTOMOC ON)
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(Qt6 6.4 REQUIRED COMPONENTS Core Gui Qml Quick QuickControls2 QuickDialogs2 LabsSettings QmlImportScanner)
qt_standard_project_setup()

if(MSVC)
  set_property(TARGET Qt6::Core Qt6::Gui Qt6::Qml Qt6::Quick Qt6::QuickControls2
                      Qt6::QuickDialogs2 Qt6::LabsSettings
               PROPERTY MAP_IMPORTED_CONFIG_DEBUG "RELEASE")
endif()

find_package(CxxQt QUIET)
if(NOT CxxQt_FOUND)
  include(FetchContent)
  FetchContent_Declare(CxxQt
    GIT_REPOSITORY https://github.com/kdab/cxx-qt-cmake.git
    GIT_TAG 0.10.0)
  FetchContent_MakeAvailable(CxxQt)
endif()

cxx_qt_import_crate(
  MANIFEST_PATH core/Cargo.toml
  CRATES genfam_core
  LOCKED
  QT_MODULES Qt::Core Qt::Gui Qt::Qml Qt::Quick Qt::QuickControls2)

cxx_qt_import_qml_module(genfam_qml_module
  URI "fr.mmedia.genfam"
  SOURCE_CRATE genfam_core)

set(GENFAM_SOURCES cpp/main.cpp)
if(WIN32)
  list(APPEND GENFAM_SOURCES build/genfam.rc)
  # ureq / rustls s'appuient sur les API réseau et cryptographiques de Windows.
  set(GENFAM_LIBS_SYSTEME ws2_32 userenv bcrypt ntdll advapi32 crypt32 secur32)
endif()

qt_add_executable(genfam ${GENFAM_SOURCES})
qt_add_resources(genfam "ressources" PREFIX "/" FILES assets/genfam.png)
target_link_libraries(genfam PRIVATE genfam_qml_module Qt::Quick Qt::QuickControls2
                                     Qt::QuickDialogs2 Qt::LabsSettings ${GENFAM_LIBS_SYSTEME})
target_compile_definitions(genfam PRIVATE GENFAM_VERSION="${PROJECT_VERSION}")

set_target_properties(genfam PROPERTIES WIN32_EXECUTABLE ON)
qt_import_qml_plugins(genfam)

install(TARGETS genfam RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR})

enable_testing()
add_test(NAME smoke COMMAND genfam --smoke ${CMAKE_SOURCE_DIR}/core/tests/donnees/exemple.ged)
set_tests_properties(smoke PROPERTIES ENVIRONMENT "QT_QPA_PLATFORM=offscreen")

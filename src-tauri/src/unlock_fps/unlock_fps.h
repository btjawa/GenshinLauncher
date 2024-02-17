#include <Windows.h>
#include <TlHelp32.h>
#include <vector>
#include <string>
#include <cstdarg>
#include <cstdio>
#include <thread>
#include <Psapi.h>

extern "C" __declspec(dllexport) uintptr_t calc_fps_offset(
    MODULEENTRY32 hUnityPlayer, MODULEENTRY32 hUserAssembly, HANDLE process
);
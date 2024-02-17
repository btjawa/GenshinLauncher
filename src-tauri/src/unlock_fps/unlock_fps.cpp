#include "unlock_fps.h"

uintptr_t PatternScan(void* module, const char* signature) {
    static auto pattern_to_byte = [](const char* pattern) {
        auto bytes = std::vector<int>{};
        auto start = const_cast<char*>(pattern);
        auto end = const_cast<char*>(pattern) + strlen(pattern);
        for (auto current = start; current < end; ++current) {
            if (*current == '?') {
                ++current;
                if (*current == '?') ++current;
                bytes.push_back(-1);
            }
            else bytes.push_back(strtoul(current, &current, 16));
        }
        return bytes;
    };
    auto dosHeader = (PIMAGE_DOS_HEADER)module;
    auto ntHeaders = (PIMAGE_NT_HEADERS)((std::uint8_t*)module + dosHeader->e_lfanew);
    auto patternBytes = pattern_to_byte(signature);
    auto scanBytes = reinterpret_cast<std::uint8_t*>(module);
    auto s = patternBytes.size();
    auto d = patternBytes.data();
    for (auto i = 0ul; i < ntHeaders->OptionalHeader.SizeOfImage - s; ++i) {
        bool found = true;
        for (auto j = 0ul; j < s; ++j) {
            if (scanBytes[i + j] != d[j] && d[j] != -1) {
                found = false;
                break;
            }
        }
        if (found) return (uintptr_t)&scanBytes[i];
    }
    return 0;
}

uintptr_t calc_fps_offset(
    MODULEENTRY32 hUnityPlayer, MODULEENTRY32 hUserAssembly, HANDLE process
) {
    LPVOID up = VirtualAlloc(nullptr, hUnityPlayer.modBaseSize + hUserAssembly.modBaseSize, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);
    if (!up) return 1;
    ReadProcessMemory(process, hUnityPlayer.modBaseAddr, up, hUnityPlayer.modBaseSize, nullptr);
    LPVOID ua = (LPVOID)((uintptr_t)up + hUnityPlayer.modBaseSize);
    ReadProcessMemory(process, hUserAssembly.modBaseAddr, ua, hUserAssembly.modBaseSize, nullptr);
    uintptr_t address = PatternScan(ua, "B9 3C 00 00 00 FF 15");
    uintptr_t pfps = 0;
    uintptr_t rip = address;
    rip += 5;
    rip += *(int32_t*)(rip + 2) + 6;
    uintptr_t ptr = 0;
    uintptr_t data = rip - (uintptr_t)ua + (uintptr_t)hUserAssembly.modBaseAddr;
    while (!ptr) {
        ReadProcessMemory(process, (LPCVOID)data, &ptr, sizeof(uintptr_t), nullptr);
        std::this_thread::sleep_for(std::chrono::milliseconds(100));
    }
    rip = ptr - (uintptr_t)hUnityPlayer.modBaseAddr + (uintptr_t)up;
    while (*(uint8_t*)rip == 0xE8 || *(uint8_t*)rip == 0xE9)
        rip += *(int32_t*)(rip + 1) + 5;
    pfps = rip + *(int32_t*)(rip + 2) + 6;
    pfps -= (uintptr_t)up;
    VirtualFree(up, 0, MEM_RELEASE);
    return pfps;
}
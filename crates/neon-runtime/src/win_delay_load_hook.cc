/*
 * When this file is linked to a DLL, it sets up a delay-load hook that
 * intervenes when the DLL is trying to load 'node.exe' or 'iojs.exe'
 * dynamically. Instead of trying to locate the .exe file it'll just return
 * a handle to the process image.
 *
 * This allows compiled addons to work when node.exe or iojs.exe is renamed.
 */

 
#include <windows.h>

#include <delayimp.h>
#include <string.h>

FARPROC WINAPI load_exe_hook(unsigned int event, DelayLoadInfo* info) {
  HMODULE m;
  if (event != dliNotePreLoadLibrary)
    return NULL;

  if (_stricmp(info->szDll, "node.exe") != 0)
    return NULL;

	
  m = GetModuleHandle(NULL);
  return (FARPROC) m;
}

decltype(__pfnDliNotifyHook2) __pfnDliNotifyHook2 = load_exe_hook;
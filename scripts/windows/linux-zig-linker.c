#include <process.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(int argc, char **argv) {
    const char *zig = getenv("FFTA_ZIG_EXE");
    const char *lib = getenv("FFTA_LINUX_LIB_DIR");
    if (!zig || !lib) {
        fputs("Missing FFTA cross-compiler environment\n", stderr);
        return 2;
    }
    char *libarg = malloc(strlen(lib) + 3);
    sprintf(libarg, "-L%s", lib);
    char **args = calloc((size_t)argc + 6, sizeof(char *));
    int n = 0;
    args[n++] = (char *)zig;
    args[n++] = "cc";
    args[n++] = "-target";
    args[n++] = "x86_64-linux-gnu.2.17";
    args[n++] = libarg;
    for (int i = 1; i < argc; i++) {
        if (strncmp(argv[i], "--target=", 9) == 0) continue;
        args[n++] = argv[i];
    }
    args[n] = NULL;
    int result = _spawnv(_P_WAIT, zig, (const char * const *)args);
    if (result == -1) perror("zig spawn");
    free(args);
    free(libarg);
    return result;
}

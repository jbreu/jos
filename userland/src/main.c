#include "libc.h"
#define DOOM_IMPLEMENTATION 
#include "PureDOOM.h"

char* mini_getenv(const char* var) {
    return ".";
}

void mini_print(const char* str) {
    write(1, str, strlen(str));
}

void mini_exit(int i) {}

void mini_get_time(int* sec, int* usec) {
    *sec = 0;
    *usec = 0;
}

__attribute__((force_align_arg_pointer))
void _start() {
    mini_print("Hallo Carina\n");

    int a = getpid();

    doom_set_gettime(mini_get_time);
    doom_set_getenv(mini_getenv);
    doom_set_print(mini_print);
    doom_set_exit(mini_exit);
    doom_set_malloc(malloc, free);
    doom_set_file_io(fopen,
                      fclose,
                      fread,
                      fwrite,
                      fseek,
                      ftell,
                      feof);

    const char * argv[] = {
    "main",
    "-shdev",
    };

    doom_init(2, argv, 0);
    while (true)
    {
        doom_update();
    }
}
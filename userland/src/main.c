#include "libc.h"
#include <inttypes.h> 
#include <stdbool.h>
#include "PureDOOM.h"

char* mini_getenv(const char* var) {
    write(1, "getenv: ", strlen("getenv: "));
    write(1, var, strlen(var));
    return ".";
}

void mini_print(const char* str) {
    write(1, str, strlen(str));
}

void mini_exit(int i) {}

__attribute__((force_align_arg_pointer))
void _start() {
    mini_print("Hallo Carina\n");

    int a = getpid();

    doom_set_gettime(get_time);
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

    char * argv[] = {
    "main",
    "-shdev",
    };

    doom_init(2, argv, 0);
    switch_vga_mode(true);

    while (true)
    {
        doom_update();
        draw_framebuffer(doom_get_framebuffer(1));

        if (get_keystate(0)) {
            doom_key_down(DOOM_KEY_UP_ARROW);
        } else {
            doom_key_up(DOOM_KEY_UP_ARROW);
        }
        
        if (get_keystate(1)) {
            doom_key_down(DOOM_KEY_LEFT_ARROW);
        } else {
            doom_key_up(DOOM_KEY_LEFT_ARROW);
        }

        if (get_keystate(2)) {
            doom_key_down(DOOM_KEY_DOWN_ARROW);
        } else {
            doom_key_up(DOOM_KEY_DOWN_ARROW);
        }

        if (get_keystate(3)) {
            doom_key_down(DOOM_KEY_RIGHT_ARROW);
        } else {
            doom_key_up(DOOM_KEY_RIGHT_ARROW);
        }

        if (get_keystate(4)) {
            doom_key_down(DOOM_KEY_CTRL);
        } else {
            doom_key_up(DOOM_KEY_CTRL);
        }

        if (get_keystate(5)) {
            doom_key_down(DOOM_KEY_SPACE);
        } else {
            doom_key_up(DOOM_KEY_SPACE);
        }

        if (get_keystate(6)) {
            doom_key_down(DOOM_KEY_ENTER);
        }
    }
}
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "original/cJSON/cJSON.h"

#ifdef _WIN32
#include <windows.h>
static double now_us(void) {
    static LARGE_INTEGER freq;
    static int have_freq = 0;
    if (!have_freq) {
        QueryPerformanceFrequency(&freq);
        have_freq = 1;
    }
    LARGE_INTEGER counter;
    QueryPerformanceCounter(&counter);
    return (double)counter.QuadPart * 1e6 / (double)freq.QuadPart;
}
#else
#include <time.h>
static double now_us(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (double)ts.tv_sec * 1e6 + (double)ts.tv_nsec / 1e3;
}
#endif

/* Reads the entire input from stdin rather than argv. A command-line
 * argument has a hard length limit on Windows (a few KB up to ~32K
 * depending on context) that JSON test/fuzz/benchmark inputs can easily
 * exceed -- a 45KB or 478KB fixture failed outright with `os error 206`
 * (ERROR_FILENAME_EXCED_RANGE) before this fix. Stdin has no such limit
 * on any platform this needs to run on. */
static char *read_all_stdin(void) {
    size_t capacity = 65536;
    size_t length = 0;
    char *buffer = (char *)malloc(capacity);
    if (!buffer) {
        return NULL;
    }

    size_t n;
    while ((n = fread(buffer + length, 1, capacity - length, stdin)) > 0) {
        length += n;
        if (length == capacity) {
            capacity *= 2;
            char *grown = (char *)realloc(buffer, capacity);
            if (!grown) {
                free(buffer);
                return NULL;
            }
            buffer = grown;
        }
    }

    buffer[length] = '\0';
    return buffer;
}

static int run_default(const char *input) {
    cJSON *root = cJSON_Parse(input);
    if (!root) {
        fprintf(stderr, "parse_error\n");
        return 1;
    }

    char *printed = cJSON_PrintUnformatted(root);
    if (!printed) {
        cJSON_Delete(root);
        return 1;
    }

    printf("%s", printed);
    cJSON_free(printed);
    cJSON_Delete(root);
    return 0;
}

/* --bench mode: times parse, formatted print, and unformatted print
 * separately, and prints machine-readable microsecond timings.
 *
 * Uses a monotonic high-resolution timer (QueryPerformanceCounter on
 * Windows, clock_gettime(CLOCK_MONOTONIC) elsewhere) instead of clock().
 * clock()'s resolution on Windows is tied to the system tick (commonly
 * ~15.6ms), so any operation faster than that -- which is every operation
 * here, all measured in microseconds -- read back as a flat 0.000us. That
 * previously made every "c(...)" timing in the benchmark meaningless on
 * Windows; this is a real accuracy fix, not just a formatting change. */
static int run_bench(const char *input) {
    double parse_start = now_us();
    cJSON *root = cJSON_Parse(input);
    double parse_end = now_us();

    if (!root) {
        fprintf(stderr, "parse_error\n");
        return 1;
    }

    double pretty_start = now_us();
    char *pretty = cJSON_Print(root);
    double pretty_end = now_us();

    double compact_start = now_us();
    char *compact = cJSON_PrintUnformatted(root);
    double compact_end = now_us();

    printf(
        "parse_us=%.3f pretty_us=%.3f compact_us=%.3f\n",
        parse_end - parse_start,
        pretty_end - pretty_start,
        compact_end - compact_start
    );

    if (pretty) {
        cJSON_free(pretty);
    }
    if (compact) {
        cJSON_free(compact);
    }
    cJSON_Delete(root);
    return 0;
}

int main(int argc, char **argv) {
    char *input = read_all_stdin();
    if (!input) {
        fprintf(stderr, "failed to read stdin\n");
        return 1;
    }

    int bench_mode = (argc >= 2 && strcmp(argv[1], "--bench") == 0);

    int rc = bench_mode ? run_bench(input) : run_default(input);
    free(input);
    return rc;
}

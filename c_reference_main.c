#include <stdio.h>
#include <string.h>
#include <time.h>
#include "original/cJSON/cJSON.h"

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
 * separately using clock(), and prints machine-readable microsecond
 * timings. This exists so bench_main (Rust side) can get a real C-side
 * timing breakdown to compare against, rather than only measuring
 * process-spawn-inclusive wall time from the Rust side. */
static int run_bench(const char *input) {
    clock_t parse_start = clock();
    cJSON *root = cJSON_Parse(input);
    clock_t parse_end = clock();

    if (!root) {
        fprintf(stderr, "parse_error\n");
        return 1;
    }

    clock_t pretty_start = clock();
    char *pretty = cJSON_Print(root);
    clock_t pretty_end = clock();

    clock_t compact_start = clock();
    char *compact = cJSON_PrintUnformatted(root);
    clock_t compact_end = clock();

    double parse_us = 1e6 * (double)(parse_end - parse_start) / CLOCKS_PER_SEC;
    double pretty_us = 1e6 * (double)(pretty_end - pretty_start) / CLOCKS_PER_SEC;
    double compact_us = 1e6 * (double)(compact_end - compact_start) / CLOCKS_PER_SEC;

    printf("parse_us=%.3f pretty_us=%.3f compact_us=%.3f\n", parse_us, pretty_us, compact_us);

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
    if (argc < 2) {
        return 1;
    }

    const char *input = argv[1];
    int bench_mode = (argc >= 3 && strcmp(argv[2], "--bench") == 0);

    if (bench_mode) {
        return run_bench(input);
    }
    return run_default(input);
}

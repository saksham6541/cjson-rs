#include <stdio.h>
#include <string.h>
#include "original/cJSON/cJSON.h"

int main(int argc, char **argv) {
    if (argc < 2) {
        return 1;
    }

    const char *input = argv[1];
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

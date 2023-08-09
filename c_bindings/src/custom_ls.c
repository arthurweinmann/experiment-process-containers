// #include "apue.h"
#include <dirent.h>
#include <stddef.h> // NULL identifier is defined inside
#include <stdio.h>
#include "error.h"

void customls(const char *name)
{
    DIR *dp;
    struct dirent *dirp;

    if ((dp = opendir(name)) == NULL)
        err_sys("can’t open %s", name);

    while ((dirp = readdir(dp)) != NULL)
        printf("%s\n", dirp->d_name);

    closedir(dp);
}
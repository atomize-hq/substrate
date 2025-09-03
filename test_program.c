#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <fcntl.h>

int main() {
    printf("Test program starting\n");
    
    // Test file operations
    int fd = open("/tmp/test_file.txt", O_CREAT | O_WRONLY | O_TRUNC, 0644);
    if (fd >= 0) {
        write(fd, "Hello from test program\n", 24);
        close(fd);
        printf("File created and written\n");
    }
    
    // Test exec
    system("echo 'Nested command via system()'");
    
    printf("Test program complete\n");
    return 0;
}
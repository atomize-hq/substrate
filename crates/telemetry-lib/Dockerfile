FROM ubuntu:22.04

# Install build tools and dependencies
RUN apt-get update && apt-get install -y \
    gcc \
    libc6-dev \
    file \
    strace \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create working directory
WORKDIR /substrate

# Copy the built telemetry library
COPY target/release/libsubstrate_telemetry.so /usr/lib/substrate/telemetry.so

# Build a simple test program inside the image
RUN echo '#include <stdio.h>\n\
#include <stdlib.h>\n\
#include <unistd.h>\n\
#include <fcntl.h>\n\
int main() {\n\
    printf("Test program starting\\n");\n\
    int fd = open("/tmp/test_file.txt", O_CREAT | O_WRONLY | O_TRUNC, 0644);\n\
    if (fd >= 0) {\n\
        write(fd, "Hello from test program\\n", 24);\n\
        close(fd);\n\
        printf("File created and written\\n");\n\
    }\n\
    system("echo \"Nested command via system()\"");\n\
    printf("Test program complete\\n");\n\
    return 0;\n\
}\n' > /tmp/test.c && \
    gcc -o /substrate/test_program /tmp/test.c && \
    rm -f /tmp/test.c

# Create test script
RUN echo '#!/bin/bash\n\
echo "=== Testing Substrate Telemetry Library in Docker ==="\n\
echo\n\
export SUBSTRATE_SESSION_ID="docker_test_$(date +%s)"\n\
export SUBSTRATE_TRACE_LOG="/tmp/trace.jsonl"\n\
export SUBSTRATE_AGENT_ID="docker_test"\n\
export SUBSTRATE_POLICY_ID="test_policy"\n\
echo "Session ID: $SUBSTRATE_SESSION_ID"\n\
echo "Trace log: $SUBSTRATE_TRACE_LOG"\n\
echo\n\
echo "Test 1: Simple commands"\n\
LD_PRELOAD=/usr/lib/substrate/telemetry.so ls /tmp > /dev/null 2>&1\n\
LD_PRELOAD=/usr/lib/substrate/telemetry.so echo "Hello from Docker"\n\
echo\n\
echo "Test 2: File operations"\n\
LD_PRELOAD=/usr/lib/substrate/telemetry.so sh -c "echo test > /tmp/test.txt && rm /tmp/test.txt"\n\
echo\n\
echo "Test 3: Custom test program"\n\
LD_PRELOAD=/usr/lib/substrate/telemetry.so /substrate/test_program\n\
echo\n\
echo "=== Results ==="\n\
if [ -f "$SUBSTRATE_TRACE_LOG" ]; then\n\
    echo "Events captured: $(wc -l < $SUBSTRATE_TRACE_LOG)"\n\
    echo "Sample events:"\n\
    head -5 $SUBSTRATE_TRACE_LOG\n\
else\n\
    echo "No trace log found"\n\
fi\n' > /substrate/test.sh && chmod +x /substrate/test.sh

CMD ["/substrate/test.sh"]

#include <stddef.h>
#include <unistd.h>

/*
 * This disposable test library has one deliberately harmless constructor signal. The sealed
 * runner injects it only into the exact alternate-path coordinator using DYLD_INSERT_LIBRARIES
 * and supplies only the protocol's fixed marker FD 4. Hardened Runtime plus library validation
 * must prevent this constructor from running, so a successful control observes an exact empty
 * marker stream. If the platform ever loads the library, this fixed marker makes the control fail
 * closed without touching files, Keychain state, or any target predicate.
 */
static const char kBenignInjectionLoadedV2[] =
    "SUBSTRATE_R3_BENIGN_INJECTION_LOADED_V2\n";

__attribute__((constructor)) static void substrate_r3_benign_injection_probe_v2(void) {
    const char *cursor = kBenignInjectionLoadedV2;
    size_t remaining = sizeof(kBenignInjectionLoadedV2) - 1;
    while (remaining != 0) {
        const ssize_t written = write(4, cursor, remaining);
        if (written <= 0) {
            return;
        }
        cursor += (size_t)written;
        remaining -= (size_t)written;
    }
}

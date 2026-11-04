/**
 * OPURE Swarm Native C99 Header-Only SDK
 * 
 * Provides native C/C++ access to OPURE autonomous agent spawning,
 * message passing across unmanaged ring buffers, and on-the-fly ternary RSI mutation.
 */

#ifndef OPURE_SWARM_H
#define OPURE_SWARM_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
    OPURE_SWARM_ROLE_TELEMETRY = 0,
    OPURE_SWARM_ROLE_COMPOSITOR = 1,
    OPURE_SWARM_ROLE_LIFTER = 2,
    OPURE_SWARM_ROLE_SENTINEL = 3,
    OPURE_SWARM_ROLE_CUSTOM = 4
} opure_swarm_role_t;

typedef struct {
    uint32_t agent_id;
    uint32_t role;
    uint64_t cap_rights;
    uint32_t morton_start;
    uint32_t morton_end;
    uint64_t signature_pos;
    uint64_t signature_neg;
} opure_swarm_descriptor_t;

/**
 * Spawn a new native micro-agent into an unmanaged OZTAE enclave.
 */
int32_t opure_swarm_spawn_c(
    const opure_swarm_descriptor_t* desc,
    uint32_t* out_agent_id
);

/**
 * Dispatch an autonomous discrete step across all registered swarm agents.
 */
uint32_t opure_swarm_tick_c(void);

#ifdef __cplusplus
}
#endif

#endif /* OPURE_SWARM_H */

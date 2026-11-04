/**
 * OPURE OS - Native C99 Hardware & Kernel ABI
 * Zero-allocation binary event bus definitions and NCM vector interfaces.
 */

#ifndef OPURE_ABI_H
#define OPURE_ABI_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

#define OPURE_EVENT_VECTOR_DIM 64
#define OPURE_ALIGN_64 __attribute__((aligned(64)))

typedef enum {
    OPURE_EVENT_KERNEL_HOOK      = 0x0001,
    OPURE_EVENT_PROCESS_FAULT    = 0x0002,
    OPURE_EVENT_MEMORY_PRESSURE  = 0x0003,
    OPURE_EVENT_IO_BOTTLENECK    = 0x0004,
    OPURE_EVENT_THERMAL_SPIKE    = 0x0005,
    OPURE_EVENT_USER_INPUT       = 0x0006,
    OPURE_EVENT_AGENT_HEARTBEAT  = 0x0007
} opure_event_kind_t;

typedef enum {
    OPURE_INTENT_NOMINAL_STEADY_STATE    = 0x00,
    OPURE_INTENT_MEMORY_PRESSURE_EVICT   = 0x01,
    OPURE_INTENT_PROCESS_HANG_DEADLOCK   = 0x02,
    OPURE_INTENT_THERMAL_THROTTLE_CORE   = 0x03,
    OPURE_INTENT_IO_QUEUE_STARVATION     = 0x04,
    OPURE_INTENT_SECURITY_PROBE          = 0x05,
    OPURE_INTENT_USER_INPUT_URGENT       = 0x06,
    OPURE_INTENT_COREDUMP_AUTO_PATCH     = 0x07,
    OPURE_INTENT_CHAOTIC_DIVERGENCE      = 0x08,
    OPURE_INTENT_ENCLAVE_QUARANTINE_LOCK = 0x09
} opure_intent_t;

/**
 * Binary Event Frame matching strict 64-byte alignment and C-ABI.
 */
typedef struct OPURE_ALIGN_64 {
    uint64_t timestamp_ns;
    uint32_t source_pid;
    uint16_t kind;
    uint16_t flags;
    uint32_t reserved;
    int8_t   feature_vector[OPURE_EVENT_VECTOR_DIM];
} opure_event_t;

/**
 * Discrete Scottish Voice Trigger Table
 */
static const char* const OPURE_SCOTTISH_TRIGGERS[] = {
    [OPURE_INTENT_NOMINAL_STEADY_STATE]  = "A' is braw, laddie! System's purrin' smooth as single malt.",
    [OPURE_INTENT_MEMORY_PRESSURE_EVICT] = "Hold yer horses! Memory's fair burstin', evictin' cache oot the windae!",
    [OPURE_INTENT_PROCESS_HANG_DEADLOCK] = "Och nay! Threads are stuck in a right proper fankle! Slicin' the lock!",
    [OPURE_INTENT_THERMAL_THROTTLE_CORE] = "Jings, crivens! Silicon is glowin' red hot! Throttlin' the core back!",
    [OPURE_INTENT_IO_QUEUE_STARVATION]   = "Cannae push bytes to disk, the queue pipes are pure jammed!",
    [OPURE_INTENT_SECURITY_PROBE]        = "Halt! Wha's this wee rogue pokin' where he disnae belong? Banished!",
    [OPURE_INTENT_USER_INPUT_URGENT]     = "Right ye are, guv! Shiftin' a' horsepower tae yer task this instant!",
    [OPURE_INTENT_COREDUMP_AUTO_PATCH]   = "Great Scott! Caught a crash in mid-air! Splicin' the trampoline on the fly!",
    [OPURE_INTENT_CHAOTIC_DIVERGENCE]    = "Hoots mon! Phase trajectory's jumped the Mandelbrot fence! Calmin' the chaos!",
    [OPURE_INTENT_ENCLAVE_QUARANTINE_LOCK] = "Lockit doon! Zero-trust enclave's clappit the irons on a rogue trajectory!"
};

/**
 * OZTAE Audit Ledger C99 Verification Interface
 */
int32_t opure_oztae_audit_verify_c(const void* frame_ptr, uint64_t expected_prev_digest);

/**
 * RSI Ternary Centroid Adaptation C99 Interface
 */
int32_t opure_rsi_mutate_centroid_c(void* centroid_ptr, uint32_t mutation_mask, uint32_t mode);

#ifdef __cplusplus
}
#endif

#endif /* OPURE_ABI_H */

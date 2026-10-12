/** @file
 * @brief Portable MICOR noise and VOX hold descriptor ABI.
 */
#ifndef RPTADV_SQUELCH_H
#define RPTADV_SQUELCH_H

#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define RPTADV_SQUELCH_ABI_VERSION 1U /**< Initial squelch descriptor ABI. */

/** @brief Opaque state for one MICOR comparator and one VOX hold timer. */
struct rptadv_squelch;

/** @brief Immutable function table for the process lifetime. */
struct rptadv_squelch_descriptor {
  uint32_t struct_size; /**< Size of this descriptor. */
  uint32_t abi_version; /**< Must equal RPTADV_SQUELCH_ABI_VERSION. */
  const char *capability_name; /**< Stable `rptadv.squelch` identity. */

  /** @brief Allocate zeroed detector state during setup. */
  struct rptadv_squelch *(*create)(void);
  /** @brief Release state after every caller has stopped. Null is accepted. */
  void (*destroy)(struct rptadv_squelch *handle);
  /** @brief Advance one validated native discriminator sample; return closed. */
  bool (*noise_update)(struct rptadv_squelch *handle, bool previous_closed,
                       double sample_power, uint32_t open_level,
                       uint32_t hysteresis);
  /** @brief Advance to an increasing in-span offset and reset hold if qualified. */
  void (*vox_advance)(struct rptadv_squelch *handle, uint32_t native_offset,
                      bool qualified, uint32_t hang_frames);
  /** @brief Consume trailing native frames and return whether hold remains. */
  bool (*vox_finish)(struct rptadv_squelch *handle, uint32_t native_frames);
};

/** @brief Return the immutable descriptor; validate identity, size and ABI. */
const struct rptadv_squelch_descriptor *rptadv_squelch_descriptor(void);

#ifdef __cplusplus
}
#endif

#endif

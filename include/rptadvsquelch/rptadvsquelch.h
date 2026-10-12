/** @file
 * @brief Portable MICOR noise and VOX hold descriptor ABI.
 */
#ifndef RPTADV_SQUELCH_H
#define RPTADV_SQUELCH_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define RPTADV_SQUELCH_ABI_VERSION 2U /**< Status/out-handle and u32-boolean ABI. */
#define RPTADV_SQUELCH_OK 0 /**< Operation completed. */
#define RPTADV_SQUELCH_INVALID_ARGUMENT -1 /**< Null required output slot. */

/** @brief Opaque state for one MICOR comparator and one VOX hold timer. */
struct rptadv_squelch;

/** @brief Immutable function table for the process lifetime. */
struct rptadv_squelch_descriptor {
  uint32_t struct_size; /**< Size of this descriptor. */
  uint32_t abi_version; /**< Must equal RPTADV_SQUELCH_ABI_VERSION. */
  const char *capability_name; /**< Stable `rptadv.squelch` identity. */

  /** @brief Allocate zeroed detector state during setup.
   * @return RPTADV_SQUELCH_OK or RPTADV_SQUELCH_INVALID_ARGUMENT.
   * A valid out slot is cleared before allocation and remains null on error.
   */
  int32_t (*create)(struct rptadv_squelch **out);
  /** @brief Release state after every caller has stopped. Null is accepted. */
  void (*destroy)(struct rptadv_squelch *handle);
  /** @brief Advance one validated native discriminator sample; return 1 when closed.
   * Boolean input is zero=false, any nonzero=true; output is normalized 0/1.
   */
  uint32_t (*noise_update)(struct rptadv_squelch *handle, uint32_t previous_closed,
                           double sample_power, uint32_t open_level,
                           uint32_t hysteresis);
  /** @brief Advance to an increasing in-span offset and reset hold if qualified.
   * qualified is zero=false, any nonzero=true.
   */
  void (*vox_advance)(struct rptadv_squelch *handle, uint32_t native_offset,
                      uint32_t qualified, uint32_t hang_frames);
  /** @brief Consume trailing native frames and return normalized 0/1 hold. */
  uint32_t (*vox_finish)(struct rptadv_squelch *handle, uint32_t native_frames);
};

/** @brief Return the immutable descriptor; validate identity, size and ABI. */
const struct rptadv_squelch_descriptor *rptadv_squelch_descriptor(void);

#ifdef __cplusplus
}
#endif

#endif

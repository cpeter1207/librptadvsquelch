#include "rptadvsquelch/rptadvsquelch.h"

#include <assert.h>
#include <stddef.h>
#include <string.h>

#if RPTADV_SQUELCH_ABI_VERSION != 2U
#error "squelch ABI 2 is required"
#endif

int main(void) {
  const struct rptadv_squelch_descriptor *api = rptadv_squelch_descriptor();
  assert(api == rptadv_squelch_descriptor());
  assert(api->struct_size == sizeof(*api));
  assert(api->abi_version == RPTADV_SQUELCH_ABI_VERSION);
  assert(strcmp(api->capability_name, "rptadv.squelch") == 0);
  assert(api->create && api->destroy && api->noise_update && api->vox_advance &&
         api->vox_finish);

  assert(api->create(NULL) == RPTADV_SQUELCH_INVALID_ARGUMENT);
  struct rptadv_squelch *probe = NULL;
  assert(api->create(&probe) == RPTADV_SQUELCH_OK && probe);
  assert(api->noise_update(probe, 7, 100000000.0, 7000, 500) == 1);
  api->destroy(probe);
  struct rptadv_squelch *state = NULL;
  assert(api->create(&state) == RPTADV_SQUELCH_OK && state);
  uint32_t closed = 1;
  for (int i = 0; i < 480; ++i) {
    closed = api->noise_update(state, closed, 100000000.0, 7000, 500);
    assert(closed);
  }
  for (int i = 0; i < 48000; ++i)
    closed = api->noise_update(state, closed, 100000000.0, 7000, 500);
  for (int i = 0; i < 24000; ++i)
    closed = api->noise_update(state, closed, 810000.0, 7000, 500);
  assert(closed == 0);
  for (int i = 0; i < 88; ++i)
    closed = api->noise_update(state, closed, 100000000.0, 7000, 500);
  assert(closed);

  api->vox_advance(state, 5, 7, 10);
  assert(api->vox_finish(state, 8) == 1);
  assert(api->vox_finish(state, 7) == 0);
  assert(api->noise_update(NULL, 7, 0.0, 0, 0) == 1);
  api->destroy(state);
  api->destroy(NULL);
  return 0;
}

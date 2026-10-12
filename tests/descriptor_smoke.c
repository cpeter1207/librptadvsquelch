#include "rptadvsquelch/rptadvsquelch.h"

#include <assert.h>
#include <stddef.h>
#include <string.h>

int main(void) {
  const struct rptadv_squelch_descriptor *api = rptadv_squelch_descriptor();
  assert(api == rptadv_squelch_descriptor());
  assert(api->struct_size == sizeof(*api));
  assert(api->abi_version == RPTADV_SQUELCH_ABI_VERSION);
  assert(strcmp(api->capability_name, "rptadv.squelch") == 0);
  assert(api->create && api->destroy && api->noise_update && api->vox_advance &&
         api->vox_finish);

  struct rptadv_squelch *state = api->create();
  assert(state);
  bool closed = true;
  for (int i = 0; i < 480; ++i) {
    closed = api->noise_update(state, closed, 100000000.0, 7000, 500);
    assert(closed);
  }
  for (int i = 0; i < 48000; ++i)
    closed = api->noise_update(state, closed, 100000000.0, 7000, 500);
  for (int i = 0; i < 24000; ++i)
    closed = api->noise_update(state, closed, 810000.0, 7000, 500);
  assert(!closed);
  for (int i = 0; i < 88; ++i)
    closed = api->noise_update(state, closed, 100000000.0, 7000, 500);
  assert(closed);

  api->vox_advance(state, 5, true, 10);
  assert(api->vox_finish(state, 8));
  assert(!api->vox_finish(state, 7));
  api->destroy(state);
  api->destroy(NULL);
  return 0;
}

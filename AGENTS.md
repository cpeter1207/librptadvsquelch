# librptadvsquelch development rules

## Shared rpt_advanced project baseline

This baseline applies to every production, shared-library, and workflow
repository in the rpt_advanced project. Repository-specific rules may add
constraints but must not weaken it.

Run platform-independent formatting, lint, static analysis including Cppcheck,
and Doxygen once, concurrently where independent. Run platform-dependent
build, tests, packaging, and staged-install checks on native Debian 13 amd64
and arm64. Require 100% line and branch coverage of production source on amd64
only; exclude test code. Do not add automatic Debian 12 checks or packages.
Release Debian 13 packages only. Quality checks must not rewrite source files.

Before a push, run formatting, lint, and static analysis. GitHub repeats those
fast checks on push. Every pull request runs the full quality gate and must pass
before merge. Release only a merged main revision that has already passed that
gate. Local recovery commits may follow targeted checks but are not fully
verified until the pull request gate passes.

Treat compiler warnings as errors. Remove unreachable or dead code instead of
suppressing diagnostics or excluding it from coverage. Update developer
documentation, tests, build, install, and package artifacts with an interface
change; update operator documentation before a pull request. Consumers must
dynamically link the released, versioned shared object, without a static or
vendored implementation. Preserve published ABI/API compatibility when
practical; document any incompatible SONAME/package migration.

Start and clean only project-owned, labeled test containers. Pull the required
`:latest` image, record its digest, and use it fresh for local container tests.
Keep iteration evidence under ignored `/.work/`. Never deploy to a node or
alter its configuration without explicit approval. Workflow implementations
belong in the companion repository; this repository holds thin callers only.

## Squelch boundary

This component owns only MICOR discriminator-noise comparator state and
sample-clocked VOX hold state. Qualification, envelope measurement, PCM
conversion, and receiver orchestration remain with librptadvradio. Follow
ADRs 0011 and 0022 in rpt_advanced for the versioned component boundary.

The exported interface is a C-compatible descriptor with one opaque handle.
Do not allocate, lock, block, log, or perform I/O in update functions. Use
warning-free Rustdoc for Rust source and Doxygen for the C header. Do not
modify consumers until the library has passed its gate and been released.

55555555555555555555555555aAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA0'.3

+++++++++++++++++++V

# Project storage checklist

Use this checklist before a large build and when the internal disk is running
low. Builds and ordinary validation must work with the external disk unplugged.

## Before starting a build or validation

- [ ] Check `df -h target`. Budget peak growth with `validation_unit.py --reserve-gib N`.
      The default launch floor is 5 GiB plus the declared growth.
- [ ] Reuse installed toolchains, dependency downloads, canonical Cargo outputs,
      the shared Wine prefix and one current dataset per ROM/profile. Keep the
      exact previous dataset or binary locally when a named comparison needs it.
- [ ] Declare read-only inputs with `--protect`. Put generated test copies in
      the unit's `work`. Use `--keep-path` for a retained result and `--keep-hot`
      for frequent access. The runner retires successful scratch after verification.
- [ ] Keep ROMs, BIOS, personal saves, recordings, source preimages and live or
      suspended sessions in their established locations.

## When space is low

- [ ] Measure the largest directories once with `du -xhd1 target | sort -hr`.
      Directory sizes include required data. For an actionable first estimate run
      `python3 scripts/cold_storage.py compact`.
- [ ] Run `python3 scripts/cold_storage.py compact --apply` and repeat the
      preview. Each released block belonged to a redundant local compressed file.
      New archives share automatically. At initial admission the runner also tries
      existing duplicates if the available space falls below the launch budget.
- [ ] Review remaining large outputs by their exact unit, consumers and leases.
      Search other test drivers and declared inputs for the exact prefix or dataset.
      A named Wine prefix can serve multiple projects or test units.
      Use the [cleanup skill](../../.cursor/skills/ffta-cleanup/SKILL.md) for the
      completed-only batch, host activity checks and verified local archival.
      Preserve the original result, all intermediate snapshots and diagnostic files
      of closed failures. `plan --local-diagnostic FAIL` or `MIXED` and a reviewed
      `pack` keep those files restorable from the internal disk.
- [ ] Move only approved, optional completed archives to an identified external
      volume using the [offload procedure](LOCAL_ARTIFACTS.md#moving-completed-archives-to-an-external-volume).
      Catalogs stay local. Restoring those specific archives requires that volume.
- [ ] Verify affected archives with `python3 scripts/cold_storage.py verify ORIGINAL_PATH`.
      Record released allocated blocks, remaining free space, and any deferred scopes
      with the reason. If the space budget still fails, the build waits.
- [ ] When running the archiver in a container, expose the selected originals and
      `target/cold-storage` through a common mount. Retirement uses an atomic rename.

## Cleanup on 19 November 2026

- [x] Measured the root workspace. At the start, `target` occupied 128.5 GiB.
- [x] Shared 175 duplicate compressed files on the internal disk and verified
      a completed archive and a failed diagnostic. A second preview found zero
      additional copies. Existing catalog entries retained their checksums.
- [x] Integrated automatic archive sharing, the low-space launch check and
      retention guidance into the development, Windows validation and cleanup skills.
- [x] Preserved four closed Wine environments on the internal disk, including
      three failed verifications and a mixed GUI run. Verified all four archives
      and exercised an offline restore and retirement at the original GUI path.
- [x] Reclaimed **5.61 GiB** after archive metadata and maintenance receipts.
      Duplicate payloads accounted for 2.25 GiB and Wine compression for 3.36 GiB.
      The local operation receipt is `target/storage-maintenance-20261119-gzbhv8ds/result.json`.
- [x] Reclaimed another **4.06 GiB** by archiving 14 isolated Cargo dependency
      caches, seven historical dataset copies and 248 intermediate images from five
      render folders. All 26 archives verified on the internal disk, including
      the outputs of failed, mixed and interrupted tests. The current dataset,
      the test executables and the 45 final render images stayed available.
- [x] Confirmed **9.67 GiB** recovered across the two operations and **15.08 GiB**
      free on the internal volume at the final check. The duplicate preview found
      zero further candidates. The local receipt is `target/storage-followup-20261119-a/result.json`.
- [x] With U31 deferred, reclaimed another **3.57 GiB** from two historical
  Wine prefixes, 22 isolated compiler outputs, 40 GUI caches and one failed
  E134 dataset. Verified all 65 local archives. Restored one GUI cache without
  the external volume, compared it byte for byte with its retained local copy,
  and verified its return to the catalog.
- [x] Measured **13.24 GiB** recovered across the three operations. The internal
  volume had **18.49 GiB** free. The local receipt is
  `target/storage-independent-20261119-3px405qb/result.json`.
- [ ] The suspended U31 shop replay build is deferred. After the work closes,
  review its isolated build tree (7.87 GiB at inventory, including 4.00 GiB
  of incremental compiler data).
      Identify the exact executable still needed and verify consumers before retiring
      any cache. The size is an upper bound for that review.
- [ ] Native Windows verification remains deferred. The verified host was Linux.

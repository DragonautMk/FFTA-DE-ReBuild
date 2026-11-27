55555555555555555555555555aAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA0'.3

+++++++++++++++++++V

# Project storage checklist

Use this checklist before a large build and when the internal disk is running
low. The default layout keeps build and validation outputs on the internal
disk. The [external target setup](#external-target-on-linux) is a workstation
exception that requires the selected disk for builds and validation.

## Before starting a build or validation

- [ ] Check `df -h . target out`. Budget peak growth with `validation_unit.py --reserve-gib N`.
      The default launch floor is 5 GiB plus the declared growth.
- [ ] Reuse installed toolchains, dependency downloads, canonical Cargo outputs,
      the shared Wine prefix and one current dataset per ROM/profile. Keep the
      exact previous dataset or binary locally when a named comparison needs it.
- [ ] Declare read-only inputs with `--protect`. Put generated test copies in
      the unit's `work`. Use `--keep-path` for a retained result and `--keep-hot`
      for frequent access. The runner retires successful scratch after verification.
- [ ] Keep ROMs, BIOS, personal saves, recordings, source preimages and live or
      suspended sessions in their established locations.

## External target on Linux

On the configured workstation, a udev rule matches the external disk's
filesystem UUID. Systemd mounts the disk, opens separate ext4 images for
`target` and `out`, and binds both directories at their usual checkout paths.
The target verification service checks the disk and target image UUIDs and
read-write mount options. The output mount depends on the target mount and its
own image mount; it does not have a separate UUID verification service. When the
disk is absent, protected empty mountpoints remain at both project paths and the
disk rule does not start a mount job.
[systemd device units](https://www.freedesktop.org/software/systemd/man/latest/systemd.device.html)

From the root checkout on the Linux host, check that both binds expose their
expected ext4 filesystems with `rw` in their options. The two images and the
outer disk each have their own free-space limit.

```sh
workspace_target="$PWD/target"
workspace_out="$PWD/out"
target_unit=$(systemd-escape --path --suffix=mount "$workspace_target")
out_unit=$(systemd-escape --path --suffix=mount "$workspace_out")
systemctl status "$target_unit" "$out_unit" --no-pager
findmnt --mountpoint "$workspace_target" -o TARGET,SOURCE,UUID,FSTYPE,OPTIONS
findmnt --mountpoint "$workspace_out" -o TARGET,SOURCE,UUID,FSTYPE,OPTIONS
df -h . "$workspace_target" "$workspace_out"
```

If the disk is attached and the mount failed, check the current boot's journal
for the mount units and `ffta-recomp-target-verify.service`. After correcting
the reported error, start the target bind with `sudo systemctl start "$target_unit"`
and, if necessary, the output bind with `sudo systemctl start "$out_unit"`.
The operating system assigns loop device names dynamically. Automatic output
reconnection was configured and inspected but has not been tested after reboot.

Before ejecting the disk, close build and validation processes, stop the `out`
bind and then the `target` bind, and stop both ext4 image mounts before the
outer volume mount. Use `systemctl cat "$target_unit" "$out_unit"` and their
dependency units to identify the installed paths. Stop if an unmount reports
the device is busy. A manually attached loop without autoclear must be detached
after unmounting its image.

The installed configuration consists of five mount units, the target
verification service, target and output udev rules, a target-unit drop-in that
requests the output mount, and the target tmpfiles entry. To remove the
automation, stop the mounts in the same order, remove the associated unit,
drop-in, rule and tmpfiles configuration, reload systemd with
`sudo systemctl daemon-reload`, and reload udev rules with
`sudo udevadm control --reload-rules`.

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

## Cleanup on 20 November 2026

- [x] Verified all 32 local archives from 25 closed test directories. They hold
  old executables and two generated JSON files. Seventeen files came from passing
  runs and 15 from failed runs. All 32 matched their original SHA-256 hashes.
- [x] Restored one GUI executable at its original path with the external volume
  unavailable. It matched the retained executable byte for byte and returned to
  the archive after comparison.
- [x] Reclaimed **3.24 GiB** after compression, catalog entries and receipts.
  The four rounds released **16.48 GiB** in total. The duplicate archive preview
  found zero further copies. The local receipt is
  `target/storage-historical-20261119-tcl2umci/result.json`.
- [x] Archived the superseded European E171 and E172 datasets, the failed E173
  extraction and a staged Linux debug executable. Verified all 354 files across
  the four local archives. The successful E173 retry and the datasets used by
  ongoing work stayed available.
- [x] Restored E171 and the failed E173 extraction with the external volume
  unavailable. The E171 text catalog matched its E179 counterpart byte for byte.
  All 116 files in the failed E173 generation matched the successful retry.
  Both temporary restorations retired after comparison.
- [x] Reclaimed another **1.73 GiB** after accounting for the catalog and
  receipts, bringing the five rounds to **18.22 GiB**. The internal volume
  had **9.99 GiB** free at verification. U31 and leased test folders remain
  deferred. The local receipt is `target/storage-closed-20261120-27pciska/result.json`.
- [x] Archived the closed debug-tools migration Wine prefix locally, preserving
  its mixed test outcomes. An offline restore verified all 1,949 inventory
  entries and 41 symbolic links before the temporary copy was retired.
- [x] Reclaimed another **0.84 GiB** after archive and receipt costs. The six
  rounds recovered **19.06 GiB** in total. At verification, the internal disk had
  **8.24 GiB** free. The local receipt is
  `target/storage-migration-wine-20261120-2ylghb1_/result.json`.
- [x] Archived six identical GUI cache copies from five closed test scopes,
  including the two copies from a scope with both failed and successful runs.
  Restored one cache from each outcome offline and matched them to the retained
  local reference. Recovered another **0.14 GiB** after archive and receipt
  costs, for **19.20 GiB** across all seven rounds. At verification, the internal
  disk had **8.38 GiB** free. The local receipt is
  `target/storage-gui-caches-20261120-hn6aza9j/result.json`.
- [x] Archived eight more GUI cache copies, including the caches of two failed
  runs. Offline restorations of a passing and a failed run matched the retained
  local copy. The additional net recovery was **0.19 GiB**, bringing the eight
  rounds to **19.39 GiB**. The internal disk had **8.43 GiB** free at verification.
  The local receipt is `target/storage-legacy-gui-20261120-kpjxnsrq/result.json`.
- [x] Archived the failed RVR-01 isolated compiler tree after its successful
  retry. Verified all 2,263 archive entries and compared the test executable
  directly from the local archive with an identical retained copy. This freed
  **0.65 GiB** net, bringing the nine rounds to **20.04 GiB**. The receipt is
  `target/storage-rvr01-20261120-0dlu0z0l/result.json`.
- [x] Restored the full RVR-01 tree once space became available. All 2,263
  entries, including 34 internal hard links, matched the archive inventory.
  The test executable matched the retained copy and the temporary tree was
  retired. The offline verification receipt is
  `target/storage-rvr01-20261120-0dlu0z0l/full-restore.json`.
- [x] Retired ten older Cargo incremental sessions after checking their newer
  siblings, file links, host activity and Cargo/session locks. The current
  executables, dependency caches and newer sessions were retained. Accounting
  for shared links and receipts, this released **1.02 GiB** in file blocks and
  brought ten rounds to **21.07 GiB**. Free space was **4.88 GiB** at the final
  measurement. The receipt is
  `target/local-artifact-cleanup-old-incremental-ie_ssgz7/result.json`.
- [x] Archived 18 RVR-02 dependency copies that matched the canonical Windows
  build byte for byte. The two test executables remain available. Restored and
  compared an `.rlib` and an `.rmeta` without the external volume. This
  released another **0.28 GiB** net, for **21.35 GiB** across eleven rounds.
  The receipt is `target/storage-rvr02-deps-20261120-ipn35fk6/result.json`.
